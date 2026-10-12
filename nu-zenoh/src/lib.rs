//
// Copyright (c) 2025 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use nu_protocol::{
    engine::{Call, Command, EngineState, Stack, StateWorkingSet},
    record, Example, LabeledError, PipelineData, ShellError, Signature, Span, Type, Value,
};
use zenoh::{internal::runtime::Runtime, Session, Wait};

mod call_ext2;
mod cmd;
mod conv;
mod interruptible_channel;
mod signature_ext;

#[derive(Debug, Clone)]
pub struct Config {
    pub experimental_options: bool,
    pub no_default_session: bool,
    pub include_paths: Vec<String>,
}

/// Adds extra context (e.g. aliases) as Nu source code
///
/// This should be called after [`crate::add_zenoh_context`].
pub const ZENOH_CONTEXT_EXTRAS: &[u8] = include_bytes!("nu/extras.nu");

/// Adds extra context for experimental commands and command options as Nu source code.
///
/// This should be called after [`crate::add_zenoh_context`] when experimental options are enabled.
pub const ZENOH_CONTEXT_EXPERIMENTAL_EXTRAS: &[u8] = include_bytes!("nu/experimental/extras.nu");

/// Adds all `zenoh *` commands to the given [`nu_protocol::engine::EngineState`].
pub fn add_zenoh_context(mut engine_state: EngineState, options: Config) -> EngineState {
    let delta = {
        let mut working_set = StateWorkingSet::new(&engine_state);

        let state = State::new(options.clone());

        if options.experimental_options {
            let xcmds: &[CommandFactory] = &[
                |st| Box::new(cmd::runtime::list::List::new(st.clone())),
                |st| Box::new(cmd::runtime::open::Open::new(st.clone())),
                |st| Box::new(cmd::runtime::close::Close::new(st.clone())),
                |st| Box::new(cmd::pub_::Pub::new(st.clone())),
                |st| Box::new(cmd::querier::Querier::new(st.clone())),
                |st| {
                    Box::new(cmd::liveliness::declare_token::DeclareToken::new(
                        st.clone(),
                    ))
                },
                |st| {
                    Box::new(cmd::liveliness::undeclare_token::UndeclareToken::new(
                        st.clone(),
                    ))
                },
                |st| Box::new(cmd::liveliness::get::Get::new(st.clone())),
                |st| Box::new(cmd::liveliness::sub::Sub::new(st.clone())),
                |st| Box::new(cmd::pub_::MatchingListener::new(st.clone())),
                |st| Box::new(cmd::querier::MatchingListener::new(st.clone())),
                |_| Box::new(cmd::decode::transport_msg::TransportMsg),
                |_| Box::new(cmd::decode::scouting_msg::ScoutingMsg),
                |_| Box::new(cmd::parse::Locator),
                |_| Box::new(cmd::parse::Endpoint),
                |_| Box::new(cmd::parse::Selector),
            ];
            for cmd in xcmds {
                add_decl_with_short_mirror(&mut working_set, cmd(&state));
            }
        }

        let cmds: &[CommandFactory] = &[
            |st| Box::new(cmd::put::Put::new(st.clone())),
            |st| Box::new(cmd::delete::Delete::new(st.clone())),
            |st| Box::new(cmd::get::Get::new(st.clone())),
            |st| Box::new(cmd::sub::Sub::new(st.clone())),
            |st| Box::new(cmd::zid::Zid::new(st.clone())),
            |st| Box::new(cmd::session::list::List::new(st.clone())),
            |st| Box::new(cmd::session::open::Open::new(st.clone())),
            |st| Box::new(cmd::session::close::Close::new(st.clone())),
            |st| Box::new(cmd::log_path::LogPath::new(st.clone())),
            |st| Box::new(cmd::queryable::Queryable::new(st.clone())),
            |st| Box::new(cmd::scout::Scout::new(st.clone())),
            |st| Box::new(cmd::info::Info::new(st.clone())),
            |st| Box::new(cmd::config::Config::new(st.clone())),
            |_| Box::new(cmd::keyexpr::Includes),
            |_| Box::new(cmd::keyexpr::Intersects),
            |_| Box::new(cmd::keyexpr::Autocanonize),
        ];
        for cmd in cmds {
            add_decl_with_short_mirror(&mut working_set, cmd(&state));
        }

        working_set.render()
    };

    if let Err(err) = engine_state.merge_delta(delta) {
        eprintln!("Error creating Zenoh command context: {err:?}");
    }

    engine_state
}

/// Adds the `$nuze` constant to the given engine state and stack.
pub fn add_nuze_constant(engine_state: &mut EngineState, stack: &mut Stack, options: &Config) {
    let span = Span::unknown();
    let zenoh_features = zenoh::FEATURES
        .split_whitespace()
        .map(|feature| Value::string(feature, span))
        .collect();
    let value = Value::record(
        record!(
            "zenoh-git-version" => Value::string(zenoh::GIT_VERSION, span),
            "zenoh-features" => Value::list(zenoh_features, span),
            "experimental-options-enabled" => Value::bool(options.experimental_options, span),
        ),
        span,
    );

    let var_id = {
        let mut working_set = StateWorkingSet::new(engine_state);
        let var_id = working_set.add_variable(b"nuze".to_vec(), span, Type::record(), false);
        working_set.set_variable_const_val(var_id, value.clone());
        let delta = working_set.render();

        if let Err(err) = engine_state.merge_delta(delta) {
            eprintln!("Error creating Nuze constant: {err:?}");
            return;
        }

        var_id
    };

    stack.add_var(var_id, value);
}

type CommandFactory = fn(&State) -> Box<dyn Command>;

fn add_decl_with_short_mirror(working_set: &mut StateWorkingSet<'_>, command: Box<dyn Command>) {
    let short_name = command
        .name()
        .strip_prefix("zenoh ")
        .map(|suffix| format!("z {suffix}"))
        .expect("commands registered with a short mirror must start with `zenoh `");

    working_set.add_decl(command.clone());
    working_set.add_decl(Box::new(ShortZenohCommand::new(short_name, command)));
}

/// Built-in `z ...` mirror for a `zenoh ...` command.
///
/// Nu aliases don't participate in subcommand completion as well as real
/// declarations do, so this wrapper registers short command names directly.
#[derive(Clone)]
struct ShortZenohCommand {
    name: String,
    command: Box<dyn Command>,
}

impl ShortZenohCommand {
    fn new(name: String, command: Box<dyn Command>) -> Self {
        Self { name, command }
    }
}

impl Command for ShortZenohCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn signature(&self) -> Signature {
        let mut signature = self.command.signature();
        signature.name = self.name.clone();
        signature
    }

    fn description(&self) -> &str {
        self.command.description()
    }

    fn extra_description(&self) -> &str {
        self.command.extra_description()
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        self.command.run(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        self.command.examples()
    }

    fn search_terms(&self) -> Vec<&str> {
        self.command.search_terms()
    }
}

#[derive(Clone)]
struct State {
    options: Config,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    runtimes: Arc<RwLock<HashMap<String, Runtime>>>,
}

impl State {
    const DEFAULT_SESSION_NAME: &str = "default";

    fn new(options: Config) -> Self {
        let mut sessions = HashMap::new();
        if !options.no_default_session {
            let default_session = zenoh::open(zenoh::Config::default())
                .wait()
                .expect("could not open default session");
            sessions.insert(Self::DEFAULT_SESSION_NAME.to_string(), default_session);
        }

        Self {
            options,
            sessions: Arc::new(RwLock::new(sessions)),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl State {
    pub(crate) fn with_session<F, T>(&self, name: &str, f: F) -> Result<T, LabeledError>
    where
        F: FnOnce(&Session) -> T,
    {
        let sessions = self.sessions.read().unwrap();
        let session = sessions
            .get(name)
            .ok_or_else(|| LabeledError::new(format!("session '{name}' not found")))?;
        Ok(f(session))
    }
}

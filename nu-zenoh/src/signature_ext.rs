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
use nu_protocol::{Category, Signature, SyntaxShape};

pub(crate) trait SignatureExt: Sized {
    fn config(self) -> Self;

    fn session(self) -> Self;

    fn zenoh_category(self) -> Self;

    fn publication(self, experimental_options: bool) -> Self;

    fn allowed_destination(self) -> Self;

    fn allowed_origin(self) -> Self;

    fn encoding(self) -> Self;

    fn complete(self) -> Self;

    fn keyexpr(self) -> Self;

    fn congestion_control(self, experimental_options: bool) -> Self;

    fn express(self) -> Self;

    fn priority(self) -> Self;

    fn reliable(self) -> Self;

    fn target(self) -> Self;

    fn consolidation(self) -> Self;
}

impl SignatureExt for Signature {
    fn config(self) -> Self {
        const ZENOH_VERSION: &str = "1.10.0";
        self.optional(
            "config",
            SyntaxShape::Record(vec![].into()),
            format!("Zenoh configuration object; see https://raw.githubusercontent.com/eclipse-zenoh/zenoh/refs/tags/{ZENOH_VERSION}/DEFAULT_CONFIG.json5"),
        )
    }

    fn session(self) -> Self {
        self.named(
            "session",
            SyntaxShape::String,
            "Session name (defauls to 'default')",
            Some('s'),
        )
    }

    fn zenoh_category(self) -> Self {
        const CATEGORY: &str = "Zenoh";
        self.category(Category::Custom(CATEGORY.to_string()))
    }

    fn allowed_destination(self) -> Self {
        self.named(
            "allowed-destination",
            SyntaxShape::String,
            "Allowed destination (either 'any', 'remote' or 'session-local')",
            None,
        )
    }

    fn allowed_origin(self) -> Self {
        self.named(
            "allowed-origin",
            SyntaxShape::String,
            "Allowed origin (either 'any', 'remote' or 'session-local')",
            None,
        )
    }

    fn congestion_control(self, experimental_options: bool) -> Self {
        let description = if experimental_options {
            "Congestion control (either 'drop', 'block', or unstable 'block-first')"
        } else {
            "Congestion control (either 'drop' or 'block')"
        };

        self.named("congestion-control", SyntaxShape::String, description, None)
    }

    fn reliable(self) -> Self {
        self.named(
            "reliable",
            SyntaxShape::Boolean,
            "Sets reliable transmission",
            None,
        )
    }

    fn express(self) -> Self {
        self.named(
            "express",
            SyntaxShape::Boolean,
            "Sets express transmission",
            None,
        )
    }

    fn priority(self) -> Self {
        self.named("priority", SyntaxShape::String, "Priority (0-7)", None)
    }

    fn publication(self, experimental_options: bool) -> Self {
        self.keyexpr()
            .allowed_destination()
            .congestion_control(experimental_options)
            .reliable()
            .express()
            .priority()
            .named("attachment", SyntaxShape::String, "Attachment data", None)
            .named(
                "timestamp",
                SyntaxShape::String,
                "Custom timestamp (expects the '<ZID>/<RFC3339>' format)",
                None,
            )
    }

    fn encoding(self) -> Self {
        self.named(
            "encoding",
            SyntaxShape::String,
            "Encoding (e.g., 'text/plain', 'application/json', etc)",
            None,
        )
    }

    fn complete(self) -> Self {
        self.named(
            "complete",
            SyntaxShape::Boolean,
            "Queryable completeness (complete if true and the given key expression includes the query key expression)",
            None,
        )
    }

    fn keyexpr(self) -> Self {
        self.required("keyexpr", SyntaxShape::String, "Key expression")
    }

    fn target(self) -> Self {
        self.named(
            "target",
            SyntaxShape::String,
            "Query target (either 'all', 'all-complete' or 'best-matching')",
            None,
        )
    }

    fn consolidation(self) -> Self {
        self.named(
            "consolidation",
            SyntaxShape::String,
            "Consolidation mode (either 'auto', 'latest', 'monotonic' or 'none')",
            None,
        )
    }
}

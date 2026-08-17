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
use nu_protocol::{
    ast, engine::EngineState, record, shell_error::generic::GenericError, IntoValue, Record,
    ShellError, Span, Value,
};
use zenoh::{bytes::ZBytes, query::Query};

/// Helper function to convert bytes to Nu value (string if valid UTF-8, otherwise bytes)
pub(crate) fn bytes_to_value(bytes: &ZBytes, span: nu_protocol::Span) -> Value {
    match bytes.try_to_string() {
        Ok(s) => Value::string(s, span),
        Err(_) => Value::binary(bytes.to_bytes(), span),
    }
}

#[allow(clippy::result_large_err)]
pub(crate) fn value_to_bytes(value: &Value) -> Result<ZBytes, nu_protocol::ShellError> {
    match value {
        Value::String { val, .. } => Ok(ZBytes::from(val)),
        Value::Binary { val, .. } => Ok(ZBytes::from(val.as_ref())),
        _ => Err(ShellError::Generic(GenericError::new_internal(
            "Invalid value type",
            "Value must be a String or Binary",
        ))),
    }
}

/// Helper function to convert a sample to a Nu record
pub(crate) fn sample_to_record_value(
    sample: zenoh::sample::Sample,
    span: nu_protocol::Span,
) -> Value {
    record!(
        "keyexpr" => sample.key_expr().to_string().into_value(span),
        "kind" => sample.kind().to_string().into_value(span),
        "congestion_control" => (sample.congestion_control() as u8).into_value(span),
        "priority" => (sample.priority() as u8).into_value(span),
        "reliable" => bool::from(sample.reliability()).into_value(span),
        "express" => sample.express().into_value(span),
        "attachment" => sample.attachment()
            .map(|a| bytes_to_value(a, span))
            .unwrap_or_default(),
        "payload" => bytes_to_value(sample.payload(), span),
        "timestamp" => sample.timestamp().map(|t| t.to_string_rfc3339_lossy().into_value(span)).unwrap_or_default(),
        "source_info" => sample
            .source_info()
            .map(|info| {
                record!(
                    "source_id" => record!(
                        "zid" => info.source_id().zid().to_string().into_value(span),
                        "eid" => info.source_id().eid().into_value(span),
                    ).into_value(span),
                    "source_sn" => info.source_sn().into_value(span)
                ).into_value(span)
            })
            .unwrap_or_default(),
        "encoding" => sample.encoding().to_string().into_value(span),
    ).into_value(span)
}

/// Helper function to convert a reply error to a Nu error
pub(crate) fn reply_error_to_error_value(
    reply_error: zenoh::query::ReplyError,
    span: nu_protocol::Span,
) -> Value {
    let error_msg = reply_error
        .payload()
        .try_to_string()
        .map(|s| format!("Reply error: '{s}'"))
        .unwrap_or_else(|err| format!("<Non UTF-8 error payload: {err}>"));
    Value::error(
        ShellError::Generic(GenericError::new_internal(error_msg, "")),
        span,
    )
}

/// Helper function to convert a query to a Nu record
pub(crate) fn query_to_record_value(query: &Query, span: nu_protocol::Span) -> Value {
    record!(
        "keyexpr" => query.selector().to_string().into_value(span),
        "parameters" => Record::from_iter(
            query.parameters().iter().map(|(k, v)| (k.to_string(), v.into_value(span))),
        ).into_value(span),
        "encoding" => query.encoding().map(|e| e.to_string().into_value(span)).unwrap_or_default(),
        "payload" => query.payload().map(|p| bytes_to_value(p, span)).unwrap_or_else(|| Value::nothing(span)),
        "attachment" => query.attachment().map(|a| bytes_to_value(a, span)).unwrap_or_default(),
    ).into_value(span)
}

// Copied verbatim from https://github.com/nushell/nushell/blob/d24845142873d1d415605f585f54e7e3852cf514/crates/nu-command/src/formats/to/json.rs#L113-L176
#[allow(clippy::result_large_err)]
pub(crate) fn value_to_json_value(
    engine_state: &EngineState,
    v: &Value,
    call_span: Span,
    serialize_types: bool,
) -> Result<nu_json::Value, ShellError> {
    // Copied verbatim from https://github.com/nushell/nushell/blob/d24845142873d1d415605f585f54e7e3852cf514/crates/nu-command/src/formats/to/json.rs#L178-L196
    #[allow(clippy::result_large_err)]
    fn json_list(
        engine_state: &EngineState,
        input: &[Value],
        call_span: Span,
        serialize_types: bool,
    ) -> Result<Vec<nu_json::Value>, ShellError> {
        let mut out = vec![];

        for value in input {
            out.push(value_to_json_value(
                engine_state,
                value,
                call_span,
                serialize_types,
            )?);
        }

        Ok(out)
    }

    let span = v.span();
    Ok(match v {
        Value::Bool { val, .. } => nu_json::Value::Bool(*val),
        Value::Filesize { val, .. } => nu_json::Value::I64(val.get()),
        Value::Duration { val, .. } => nu_json::Value::I64(*val),
        Value::Date { val, .. } => nu_json::Value::String(val.to_string()),
        Value::Float { val, .. } => nu_json::Value::F64(*val),
        Value::Int { val, .. } => nu_json::Value::I64(*val),
        Value::Nothing { .. } => nu_json::Value::Null,
        Value::String { val, .. } => nu_json::Value::String(val.to_string()),
        Value::Glob { val, .. } => nu_json::Value::String(val.to_string()),
        Value::CellPath { val, .. } => nu_json::Value::Array(
            val.members
                .iter()
                .map(|x| match &x {
                    ast::PathMember::String { val, .. } => Ok(nu_json::Value::String(val.clone())),
                    ast::PathMember::Int { val, .. } => Ok(nu_json::Value::U64(*val as u64)),
                })
                .collect::<Result<Vec<nu_json::Value>, ShellError>>()?,
        ),

        Value::List { vals, .. } => {
            nu_json::Value::Array(json_list(engine_state, vals, call_span, serialize_types)?)
        }
        Value::Error { error, .. } => return Err(*error.clone()),
        Value::Closure { val, .. } => {
            if serialize_types {
                let closure_string = val.coerce_into_string(engine_state, span)?;
                nu_json::Value::String(closure_string.to_string())
            } else {
                return Err(ShellError::UnsupportedInput {
                    msg: "closures are currently not deserializable (use --serialize to serialize as a string)".into(),
                    input: "value originates from here".into(),
                    msg_span: call_span,
                    input_span: span,
                });
            }
        }
        Value::Range { .. } => nu_json::Value::Null,
        Value::Binary { val, .. } => {
            nu_json::Value::Array(val.iter().map(|x| nu_json::Value::U64(*x as u64)).collect())
        }
        Value::Record { val, .. } => {
            let mut m = nu_json::Map::new();
            for (k, v) in &**val {
                m.insert(
                    k.clone(),
                    value_to_json_value(engine_state, v, call_span, serialize_types)?,
                );
            }
            nu_json::Value::Object(m)
        }
        Value::Custom { val, .. } => {
            let collected = val.to_base_value(span)?;
            value_to_json_value(engine_state, &collected, call_span, serialize_types)?
        }
    })
}

// Copied verbatim from https://github.com/nushell/nushell/blob/8229ccb7fa42cb3f3e740cd642efb6e48b5860ba/crates/nu-command/src/formats/from/json.rs#L151C1-L187C2
pub(crate) fn nujson_to_value(value: nu_json::Value, span: Span) -> Value {
    match value {
        nu_json::Value::Array(array) => Value::list(
            array
                .into_iter()
                .map(|x| nujson_to_value(x, span))
                .collect(),
            span,
        ),
        nu_json::Value::Bool(b) => Value::bool(b, span),
        nu_json::Value::F64(f) => Value::float(f, span),
        nu_json::Value::I64(i) => Value::int(i, span),
        nu_json::Value::Null => Value::nothing(span),
        nu_json::Value::Object(k) => Value::record(
            k.into_iter()
                .map(|(k, v)| (k, nujson_to_value(v, span)))
                .collect(),
            span,
        ),
        nu_json::Value::U64(u) => {
            if u > i64::MAX as u64 {
                Value::error(
                    ShellError::CantConvert {
                        to_type: "i64 sized integer".into(),
                        from_type: "value larger than i64".into(),
                        span,
                        help: None,
                    },
                    span,
                )
            } else {
                Value::int(u as i64, span)
            }
        }
        nu_json::Value::String(s) => Value::string(s, span),
    }
}

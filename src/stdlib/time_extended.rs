use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Local;
use crate::interpreter::values::*;

pub fn register(env: &mut crate::interpreter::environment::Environment) {
    // time_format(timestamp_float, pattern) -> Str
    // Pattern supports: YYYY, MM, DD, HH, mm, ss
    env.define("time_format", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_format".into(),
        arity: Some(2),
        func: Box::new(|args| {
            let ts = args.first().and_then(|v| v.as_float()).unwrap_or(0.0);
            let pattern = match args.get(1) {
                Some(Value::String(s)) => s.clone(),
                _ => "%Y-%m-%d %H:%M:%S".into(),
            };
            let fmt = pattern
                .replace("YYYY", "%Y")
                .replace("MM", "%m")
                .replace("DD", "%d")
                .replace("HH", "%H")
                .replace("mm", "%M")
                .replace("ss", "%S");
            // Convert float timestamp to chrono DateTime
            let secs = ts as i64;
            let nsecs = ((ts - secs as f64) * 1_000_000_000.0).abs() as u32;
            use chrono::TimeZone;
            let dt = chrono::Utc.timestamp_opt(secs, nsecs)
                .single()
                .unwrap_or_else(|| chrono::Utc.timestamp_opt(0, 0).single().unwrap());
            Ok(Value::String(dt.format(&fmt).to_string()))
        }),
    })));

    // time_year() -> Int
    env.define("time_year", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_year".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(Local::now().format("%Y").to_string().parse().unwrap_or(0)))
        }),
    })));

    // time_month() -> Int
    env.define("time_month", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_month".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(Local::now().format("%m").to_string().parse().unwrap_or(0)))
        }),
    })));

    // time_day() -> Int
    env.define("time_day", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_day".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(Local::now().format("%d").to_string().parse().unwrap_or(0)))
        }),
    })));

    // time_hour() -> Int
    env.define("time_hour", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_hour".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(Local::now().format("%H").to_string().parse().unwrap_or(0)))
        }),
    })));

    // time_minute() -> Int
    env.define("time_minute", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_minute".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(Local::now().format("%M").to_string().parse().unwrap_or(0)))
        }),
    })));

    // time_second() -> Int
    env.define("time_second", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_second".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::Int(Local::now().format("%S").to_string().parse().unwrap_or(0)))
        }),
    })));

    // time_weekday() -> Str (e.g. "Monday")
    env.define("time_weekday", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_weekday".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(Local::now().format("%A").to_string()))
        }),
    })));

    // time_date_str() -> Str (YYYY-MM-DD)
    env.define("time_date_str", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_date_str".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(Local::now().format("%Y-%m-%d").to_string()))
        }),
    })));

    // time_datetime_str() -> Str (YYYY-MM-DD HH:MM:SS)
    env.define("time_datetime_str", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_datetime_str".into(),
        arity: Some(0),
        func: Box::new(|_| {
            Ok(Value::String(Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
        }),
    })));

    // time_measure_start() -> Float (current timestamp in seconds as float)
    env.define("time_measure_start", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_measure_start".into(),
        arity: Some(0),
        func: Box::new(|_| {
            let dur = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            Ok(Value::Float(dur.as_secs_f64()))
        }),
    })));

    // time_measure_end(start) -> Float (elapsed milliseconds since start)
    env.define("time_measure_end", Value::NativeFunction(Rc::new(NativeFunctionValue {
        name: "time_measure_end".into(),
        arity: Some(1),
        func: Box::new(|args| {
            let start = args.first().and_then(|v| v.as_float()).unwrap_or(0.0);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64();
            let elapsed_ms = (now - start) * 1000.0;
            Ok(Value::Float(elapsed_ms))
        }),
    })));
}

use std::env;
use std::fmt;
use std::str::FromStr;

pub fn unwrap_env(name: &str) -> String {
    env::var(name).expect(format!("env '{}' variable not defined", name).as_str())
}

pub fn parse_env<T>(name: &str) -> T
where
    T: FromStr,
    T::Err: fmt::Debug,
{
    str::parse(unwrap_env(name).as_str()).expect(format!("env '{}' parse error", name).as_str())
}

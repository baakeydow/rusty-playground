//! UTILS

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::rc::Rc;

use super::by_address;
use chrono::DateTime;
use chrono::{Local, Timelike};
use regex::Regex;

/// Get unique values from hashmap
/// Usage: let urls: HashSet<String> = extract_field_from_data(dtk_pocket_data.clone(), |data: &DtkPocketData| data.url.clone());
pub fn extract_field_from_data<T: Clone + std::cmp::Eq, F: Fn(&T) -> String>(
    data: HashMap<String, T>,
    field_extractor: F,
) -> HashSet<String> {
    data.values().map(field_extractor).collect()
}

/// Remove duplicate values in data of hashmap
/// # Examples
///
/// ```
/// let mut dtk_pocket_data: HashMap<String, DtkPocketData>
/// remove_duplicate_hashmap(&mut dtk_pocket_data, "url");
/// ```
pub fn remove_duplicate_hashmap<'a, T: Clone + std::cmp::Eq + std::ops::Index<&'a str, Output = String>>(
    dtk_pocket_data: &mut HashMap<String, T>,
    key_to_collect: &'a str,
) {
    let mut unique_urls: HashSet<String> = HashSet::new();
    dtk_pocket_data.retain(|_key, pocket| {
        if unique_urls.contains(&pocket[key_to_collect]) {
            return false;
        }
        unique_urls.insert(pocket[key_to_collect].clone());
        true
    });
}

/// Transform Hashmap<String, T> into Vec<_>
/// Usage: let dtk_pocket_vec = hashmap_to_vec(&dtk_pocket_data);
pub fn hashmap_to_vec<T: Clone + std::cmp::Eq>(data: &HashMap<String, T>) -> Vec<(String, T)> {
    data.iter().map(|(key, value)| (key.clone(), value.clone())).collect()
}

/// Pring env vars
pub fn log_env_vars() {
    for (key, value) in std::env::vars() {
        log::trace!("{key}: {value}");
    }
}

/// Return Rc address as string
pub fn get_address_as_string<T>(ptr: &Rc<T>) -> String
where
    T: std::fmt::Display,
{
    let raw_ptr = by_address::ByAddress(ptr.clone()).addr();
    format!("{raw_ptr:p}")
}

/// Return null if string is empty
pub fn null_if_empty(s: String) -> Option<String> {
    Some(s)
        .map(|s| if s.is_empty() { None } else { Some(s) })
        .unwrap_or(None)
}

/// Check if a string only contains numbers
pub fn is_string_num(s: &str) -> bool {
    s.chars().all(char::is_numeric)
}

/// Print type of a variable
pub fn print_type_of<T>(_: &T) {
    println!("type is => {}", std::any::type_name::<T>())
}

/// Get file contents as String
pub fn get_file_as_string(filename: &str) -> Result<String, io::Error> {
    let mut f = File::open(filename)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

/// Double elements of an array using IndexMut & MulAssign
pub fn double(a: &mut [i32]) {
    use std::ops::IndexMut;
    use std::ops::MulAssign;

    for n in 0..a.len() {
        (*(*a).index_mut(n)).mul_assign(2);
    }
}

/// Get element from vec by types
fn get_element_from_vec_by_type(vec: Vec<String>) -> (Vec<String>, Vec<i128>, Vec<f64>) {
    let mut str_args: Vec<String> = Vec::new();
    let mut int_args: Vec<i128> = Vec::new();
    let mut float_args: Vec<f64> = Vec::new();
    if vec.len() == 0 {
        return (str_args, int_args, float_args);
    }
    for val in vec.into_iter() {
        if let Ok(i) = val.parse() {
            int_args.push(i);
        } else if let Ok(f) = val.parse() {
            float_args.push(f);
        } else {
            str_args.push(val);
        }
    }
    (str_args, int_args, float_args)
}

/// Get args by types
pub fn get_args_by_types() -> (Vec<String>, Vec<i128>, Vec<f64>) {
    let program_args: Vec<String> = env::args().collect();
    get_element_from_vec_by_type(program_args[1..].to_vec())
}

/// Log program arguments
pub fn log_args() {
    let (str_args, int_args, float_args) = get_args_by_types();
    println!("strings => {:?}", str_args);
    println!("integers => {:?}", int_args);
    println!("floats => {:?}", float_args);
}

/// Get RUSTY dev mode
pub fn is_rusty_dev() -> bool {
    std::env::var("RUSTY_DEV_MODE")
        .unwrap_or_else(|_| "false".into())
        .parse()
        .unwrap()
}

/// Format DateTime
pub fn format_datetime(date_to_fmt: DateTime<Local>) -> String {
    let (is_pm, hour) = date_to_fmt.hour12();
    let min = date_to_fmt.minute();
    let sec = date_to_fmt.second();
    let am_pm = if is_pm { "PM" } else { "AM" };
    let str = format!("{hour}:{min}:{sec} {am_pm}");
    str
}

/// Validate mongo search
pub fn is_valid_mongo_search(search: &str) -> bool {
    let re = Regex::new(r#"^[\w\s\d\-@.'?_"\-,!Éèéçà\[\]]+$"#).unwrap();
    re.is_match(search)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_mongo_search() {
        let mut search = "go ? baakey@github.com";
        assert_eq!(is_valid_mongo_search(search), true);
        search = "let's push,_ this - !";
        assert_eq!(is_valid_mongo_search(search), true);
        search = "let:s push this";
        assert_eq!(is_valid_mongo_search(search), false);
    }
}

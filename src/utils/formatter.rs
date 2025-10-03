use js_sys::{Error as JsError, Reflect};
use lazy_static::lazy_static;
use regex::Regex;
use wasm_bindgen::JsValue;

pub struct WasmTracebackFormatter;

impl WasmTracebackFormatter {
    const CONSOLE_WIDTH: usize = 80;
    const HEADER_PREFIX: &'static str = "┌─ Exception Trace ";
    const MESSAGE_PREFIX: &'static str = "│  Message: ";
    const DETAILS_PREFIX: &'static str = "│  Details: ";
    const STACK_HEADER: &'static str = "│  Stack:";
    const STACK_ITEM_PREFIX: &'static str = "│    → ";
    const STACK_LOCATION_PREFIX: &'static str = "│         at ";
    const FOOTER_PREFIX: &'static str = "└";

    pub fn new() -> Self {
        WasmTracebackFormatter
    }

    fn parse_error_line(&self, line_content: &str) -> (String, String) {
        lazy_static! {
            static ref ERROR_PATTERN: Regex = Regex::new(r"^(.*?): Error\((.*)\)$").unwrap();
        }

        if let Some(matches) = ERROR_PATTERN.captures(line_content) {
            let file_info = matches.get(1).map_or("", |m| m.as_str());
            let error_info = matches.get(2).map_or("", |m| m.as_str());

            let primary_message = self.extract_primary_message(file_info);
            let cleaned_details = self.clean_error_info(error_info);

            (primary_message, cleaned_details)
        } else {
            (line_content.trim().to_string(), "".to_string())
        }
    }

    fn extract_primary_message(&self, file_info: &str) -> String {
        if let Some(colon_position) = file_info.rfind(':') {
            let after_colon = &file_info[colon_position + 1..].trim();

            if !after_colon.is_empty() && after_colon.chars().next().unwrap().is_ascii_digit() {
                let mut message_start = colon_position + 1;
                while message_start < file_info.len()
                    && file_info
                        .chars()
                        .nth(message_start)
                        .unwrap()
                        .is_ascii_digit()
                {
                    message_start += 1;
                }
                return file_info[message_start..].trim().to_string();
            }

            after_colon.to_string()
        } else {
            file_info.trim().to_string()
        }
    }

    fn clean_error_info(&self, error_info: &str) -> String {
        lazy_static! {
            static ref QUOTE_PATTERN: Regex = Regex::new(r"`").unwrap();
        }

        let mut cleaned_info = error_info.trim();

        if cleaned_info.ends_with(')') {
            cleaned_info = &cleaned_info[..cleaned_info.len() - 1];
        }

        if cleaned_info.starts_with('"') {
            cleaned_info = &cleaned_info[1..];
        }
        if cleaned_info.ends_with('"') {
            cleaned_info = &cleaned_info[..cleaned_info.len() - 1];
        }

        QUOTE_PATTERN.replace_all(cleaned_info, "'").to_string()
    }

    fn format_stack_lines(&self, stack_lines: &[&str]) -> Vec<String> {
        lazy_static! {
            static ref STACK_PATTERN: Regex = Regex::new(r"(.+?) @ (.+)$").unwrap();
        }

        let mut formatted_output = Vec::new();

        for line in stack_lines {
            let trimmed_line = line.trim();
            if let Some(matches) = STACK_PATTERN.captures(trimmed_line) {
                let function_name = matches.get(1).map_or("", |m| m.as_str());
                let location_info = matches.get(2).map_or("", |m| m.as_str());

                formatted_output.push(format!("{}{}", Self::STACK_ITEM_PREFIX, function_name));
                formatted_output.push(format!("{}{}", Self::STACK_LOCATION_PREFIX, location_info));
            } else {
                formatted_output.push(format!("{}{}", Self::STACK_ITEM_PREFIX, trimmed_line));
            }
        }

        formatted_output
    }

    pub fn format(&self, error_message: &str) -> String {
        let js_error = JsError::new(error_message);
        let raw_stack = Reflect::get(&js_error.into(), &JsValue::from_str("stack"))
            .ok()
            .and_then(|val| val.as_string())
            .unwrap_or_else(|| "Cannot get stack".to_string());

        let combined_output = format!("{}\n{}", error_message, raw_stack);

        let lines: Vec<&str> = combined_output
            .split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if lines.is_empty() {
            return "Empty traceback provided".to_string();
        }

        let (primary_message, details) = self.parse_error_line(lines[0]);

        let mut result_lines = Vec::new();
        result_lines.push(format!(
            "{}{}",
            Self::HEADER_PREFIX,
            "─".repeat(Self::CONSOLE_WIDTH - Self::HEADER_PREFIX.len())
        ));

        result_lines.push(format!("{}{}", Self::MESSAGE_PREFIX, primary_message));

        if !details.is_empty() {
            result_lines.push(format!("{}{}", Self::DETAILS_PREFIX, details));
        }

        if lines.len() > 1 {
            result_lines.push("│".to_string());
            result_lines.push(Self::STACK_HEADER.to_string());
            result_lines.extend(self.format_stack_lines(&lines[1..]));
        }

        result_lines.push(format!(
            "{}{}",
            Self::FOOTER_PREFIX,
            "─".repeat(Self::CONSOLE_WIDTH - 1)
        ));

        result_lines.join("\n")
    }

    pub fn format_error(error_message: &str) -> String {
        let formatter = WasmTracebackFormatter::new();
        formatter.format(error_message)
    }

    pub fn log(&self, error_message: &str) {
        let formatted_output = self.format(error_message);
        web_sys::console::error_1(&JsValue::from_str(&formatted_output));
    }
}

pub fn format_wasm_traceback(error_message: &str) -> String {
    WasmTracebackFormatter::format_error(error_message)
}

pub fn log(error_message: &str) {
    let formatter = WasmTracebackFormatter::new();
    formatter.log(error_message);
}

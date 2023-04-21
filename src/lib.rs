//! This crate is for parsing command line and environment parameters.
//! It's designed to be lightweight and straightforward to use

use crate::Error::{
	DuplicateArgument, MissingArgumentValue, RequiredArgumentMissing, TooFewArguments,
	TooManyArguments,
};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::{env, fmt, process};

// Accepts a reference to a vactor of Strings and a delimeter. Returns a String with all the
// provided strings concatenated together with the delimeter string separating them
fn collect_strs(vec: &Vec<String>, delimiter: &str) -> String {
	let mut string = String::new();

	for i in 0..vec.len() {
		string.push_str(&vec[i]);
		if i < vec.len() - 1 {
			string.push_str(delimiter);
		}
	}

	string
}

pub struct Arg {
	name: String,
	short: Option<Vec<String>>,
	long: Option<Vec<String>>,
	environment: Option<String>,
	accepts_value: bool,
	required: bool,
	completed: bool,
	help: Option<String>,
	default: Option<String>,
}

impl fmt::Display for Arg {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut str = String::new();

		if let Some(short) = &self.short {
			str.push_str("  "); // We need a little bit of padding here
			str.push_str(&collect_strs(short, ","));
		}
		str.push('\t');

		if let Some(long) = &self.long {
			str.push_str(&collect_strs(long, ","));
		}
		str.push('\t');

		if self.accepts_value {
			str.push_str("(value)\t");
		}

		if let Some(help) = &self.help {
			str.push_str(help);
		}

		write!(f, "{}", &str)
	}
}

impl Arg {
	pub fn new(name: &str) -> Arg {
		Arg {
			name: name.to_string(),
			short: None,
			long: None,
			environment: None,
			accepts_value: false,
			required: false,
			completed: false,
			help: None,
			default: None,
		}
	}

	pub fn add_short(mut self, short: &str) -> Arg {
		if let Some(shorts) = &mut self.short {
			shorts.push(short.to_string());
		} else {
			self.short = Some(vec![short.to_string()]);
		}

		self
	}

	pub fn add_long(mut self, long: &str) -> Arg {
		if let Some(longs) = &mut self.long {
			longs.push(long.to_string());
		} else {
			self.long = Some(vec![long.to_string()]);
		}

		self
	}

	pub fn accepts_value(mut self) -> Arg {
		self.accepts_value = true;

		self
	}

	pub fn help(mut self, help: &str) -> Arg {
		self.help = Some(help.to_string());

		self
	}

	pub fn set_default(mut self, default: &str) -> Arg {
		self.default = Some(default.to_string());
		self.required = false;

		self
	}

	pub fn environment(mut self, env_var: &str) -> Arg {
		self.environment = Some(env_var.to_string());

		self
	}

	fn set_completed(&mut self) {
		self.completed = true;
	}

	fn is_completed(&self) -> bool {
		self.completed
	}

	fn is_done(&self) -> bool {
		self.completed || !self.required
	}

	fn matches(&self, input: &str) -> bool {
		if let Some(short) = &self.short {
			for str in short {
				if str.eq(input) {
					return true;
				}
			}
		}

		if let Some(long) = &self.long {
			for str in long {
				if str.eq(input) {
					return true;
				}
			}
		}

		false
	}
}

pub struct App {
	exec_name: String,
	pretty_name: Option<String>,
	version: Option<String>,
	author: Option<String>,
	about: Option<String>,
	args: Vec<Arg>,
	untagged_args_req: Vec<String>,
	untagged_args_opt: Vec<String>,
	manual_help_flag: bool,
	manual_version_flag: bool,
}

impl App {
	pub fn new(exec_name: &str) -> App {
		App {
			exec_name: exec_name.to_string(),
			pretty_name: None,
			version: None,
			author: None,
			about: None,
			args: Vec::new(),
			untagged_args_req: Vec::new(),
			untagged_args_opt: Vec::new(),
			manual_help_flag: false,
			manual_version_flag: false,
		}
	}

	pub fn pretty_name(&mut self, pretty_name: &str) -> &mut App {
		self.pretty_name = Some(pretty_name.to_string());

		self
	}

	pub fn version(&mut self, version: &str) -> &mut App {
		self.version = Some(version.to_string());

		self
	}

	pub fn author(&mut self, author: &str) -> &mut App {
		self.author = Some(author.to_string());

		self
	}

	pub fn about(&mut self, about: &str) -> &mut App {
		self.about = Some(about.to_string());

		self
	}

	fn is_manual_help(args: Option<&Vec<String>>) -> bool {
		if let Some(args) = args {
			for arg in args {
				let arg = arg.to_lowercase();
				if arg.eq("-help") | arg.eq("--help") {
					return true;
				}
			}
		}

		false
	}

	fn is_manual_version(args: Option<&Vec<String>>) -> bool {
		if let Some(args) = args {
			for arg in args {
				let arg = arg.to_lowercase();
				if arg.eq("-version") | arg.eq("--version") {
					return true;
				}
			}
		}

		false
	}

	pub fn arg(&mut self, arg: Arg) -> &mut App {
		// Check for the user manually setting up a help or version flag flag, though they probably wont do this as that's kinda the point of this library

		if App::is_manual_help(arg.short.as_ref()) | App::is_manual_help(arg.long.as_ref()) {
			self.manual_help_flag = true;
		}

		if App::is_manual_version(arg.short.as_ref()) | App::is_manual_version(arg.long.as_ref()) {
			self.manual_version_flag = true;
		}

		self.args.push(arg);

		self
	}

	pub fn untagged_required_arg(&mut self, name: &str) -> &mut App {
		self.untagged_args_req.push(name.to_string());

		self
	}

	pub fn untagged_optional_arg(&mut self, name: &str) -> &mut App {
		self.untagged_args_opt.push(name.to_string());

		self
	}

	fn get_arg(&mut self, name: &str) -> Option<&mut Arg> {
		self.args.iter_mut().find(|arg| arg.name.eq(name))
	}

	fn match_arg(&mut self, input: &str) -> Option<&mut Arg> {
		self.args.iter_mut().find(|arg| arg.matches(input))
	}

	fn check_arg(&self, input: &str) -> bool {
		self.args.iter().find(|arg| arg.matches(input)).is_some()
	}

	fn clear_args(&mut self) {
		for arg in self.args.iter_mut() {
			arg.completed = false;
		}
	}
}

impl App {
	fn parse_internal(
		&mut self,
		args: impl Iterator<Item = String>,
	) -> Result<HashMap<String, String>, Error> {
		let mut args = args.peekable();

		let mut output: HashMap<String, String> = HashMap::new();

		let mut untagged_req: i32 = 0;
		let mut untagged_opt: i32 = 0;

		// We're always going to put the path of the executable in here, just in case
		output.insert("path".to_string(), args.next().unwrap_or(".\\".to_string()));

		while let Some(input) = args.next() {
			if let Some(arg) = self.match_arg(&input) {
				if arg.is_completed() {
					return Err(DuplicateArgument(arg.name.clone()));
				}
				arg.set_completed(); // As far as I know, there's no reason that this shouldnt go
					 // here, no reason it should need to be at the end of this if
					 // case, but I'll leave this note here just in case it causes
					 // issues in the future
				if arg.accepts_value {
					let arg_name = arg.name.clone();
					if let Some(value) = args.peek() {
						if !self.check_arg(value) {
							output.insert(arg_name, args.next().unwrap());
						}
					} else if let Some(default) = arg.default.clone() {
						output.insert(arg_name, default);
					} else {
						return Err(MissingArgumentValue(arg_name));
					}
				} else {
					output.insert(arg.name.clone(), "true".to_string());
				}
			} else if untagged_req < self.untagged_args_req.len() as i32 {
				output.insert(self.untagged_args_req[untagged_req as usize].clone(), input);
				untagged_req += 1;
			} else if untagged_opt < self.untagged_args_opt.len() as i32 {
				output.insert(self.untagged_args_opt[untagged_opt as usize].clone(), input);
				untagged_opt += 1;
			} else {
				return Err(TooManyArguments(input));
			}
		}

		if untagged_req < (self.untagged_args_req.len() - 1) as i32 {
			return Err(TooFewArguments);
		}

		for arg in self
			.args
			.iter_mut()
			.filter(|arg| !arg.completed && arg.environment.is_some())
		{
			if env::var(
				arg.environment
					.as_ref()
					.expect("Checked during the iter filter"),
			)
			.is_ok()
			{
				if let Some(default) = &arg.default {
					output.insert(arg.name.clone(), default.clone());
				} else {
					output.insert(arg.name.clone(), "true".to_string());
				}
				arg.completed = true;
			}
		}

		let required_missing: Vec<&Arg> = self
			.args
			.iter()
			.filter(|arg| arg.required && !arg.is_done())
			.collect();

		if !required_missing.is_empty() {
			return Err(RequiredArgumentMissing(
				required_missing
					.iter()
					.map(|arg| arg.name.clone())
					.collect(),
			));
		}

		for arg in self
			.args
			.iter_mut()
			.filter(|arg| !arg.is_completed() && arg.default.is_some())
		{
			output.insert(
				arg.name.clone(),
				arg.default
					.as_ref()
					.expect("Validated in the iter filter")
					.clone(),
			);
			arg.completed = true;
		}

		self.clear_args();

		Ok(output)
	}

	pub fn parse(&mut self, args: impl Iterator<Item = String>) -> HashMap<String, String> {
		match self.parse_internal(args) {
			Ok(val) => val,
			Err(error) => {
				match error {
					TooFewArguments => {
						eprintln!("Too few arguments!\n");
						self.print_help();
						process::exit(1);
					}
					TooManyArguments(str) => {
						eprintln!("You entered too many arguments, '{str}' was unexpected.\n");
						self.print_help();
						process::exit(1);
					}
					MissingArgumentValue(name) => {
						eprintln!("User failed to provide value for the following argument: ");
						eprintln!(
							"{}\n",
							self.get_arg(&name)
								.expect("This value should come from a valid arg")
						);
						self.print_help();
						process::exit(1);
					}
					DuplicateArgument(name) => {
						eprintln!("User provided the following argument twice: ");
						eprintln!(
							"{}\n",
							self.get_arg(&name)
								.expect("This value should come from a valid arg")
						);
						self.print_help();
						process::exit(1);
					}
					RequiredArgumentMissing(names) => {
						eprintln!("User did not provide one or more of the following required arguments: ");
						for name in names {
							eprintln!(
								"{}",
								self.get_arg(&name)
									.expect("This value should come from a valid arg")
							);
						}
						eprintln!("\n");
						self.print_help();
						process::exit(1);
					}
				}
			}
		}
	}

	fn print_help(&self) {
		// If the user has provided specific text for this then we'll display that. Otherwise we'll display the name of the executable
		if let Some(name) = &self.pretty_name {
			println!("{name}");
		} else {
			println!("{}", &self.exec_name);
		}

		if let Some(version) = &self.version {
			print!("  Version: {version}")
		}
		if let Some(author) = &self.author {
			print!("\tAuthor: {author}");
		}

		println!();
		if let Some(about) = &self.about {
			println!("{about}\n");
		}

		println!("USAGE:\titems marked with * are optional\n");
		print!("{}", &self.exec_name);
		for arg in &self.untagged_args_req {
			print!(" {}", arg);
		}
		for arg in &self.untagged_args_opt {
			print!(" {}*", arg);
		}

		if !self.args.is_empty() {
			println!(" [OPTIONS]*\n");
			println!("Options: ");
		}

		// Print out each arg
		self.args.iter().for_each(|arg| {
			println!("{}", arg.to_string());
		})
	}
}

#[derive(Debug)]
enum Error {
	TooFewArguments,
	TooManyArguments(String),
	MissingArgumentValue(String),
	DuplicateArgument(String),
	RequiredArgumentMissing(Vec<String>),
}

#[cfg(test)]
mod tests {

	use super::*;

	fn generate_test_app() -> App {
		let mut app = App::new("Test");
		app.pretty_name("Test App")
			.version("1.0")
			.about("A test app to make sure everything is working for my argument parsing library")
			.author("Cameron Barnes, Cameron_barnes@outlook.com")
			.untagged_required_arg("first_input")
			.untagged_optional_arg("optional_second_input")
			.arg(
				Arg::new("TestArg")
					.add_short("-vv")
					.help("Help text for this flag"),
			)
			.arg(Arg::new("TestArg2").add_short("-f").add_long("-flag"))
			.arg(
				Arg::new("TestArg3")
					.add_short("-1")
					.add_short("-2")
					.add_short("-3")
					.add_long("-one")
					.add_long("-two")
					.help("Help info here!"),
			)
			.arg(
				Arg::new("HasValue")
					.add_short("-v")
					.add_long("-value")
					.accepts_value()
					.help("A command line option that accepts a value"),
			)
			.arg(
				Arg::new("HasDefault")
					.add_short("-d")
					.add_long("--default")
					.accepts_value()
					.set_default("Default Value")
					.help("Default Value Parameter"),
			);

		app
	}

	#[test]
	fn test_required_and_optional_argument_parsing() {
		// Build the test app
		let mut app = generate_test_app();

		// First we're going to test it with just the required and optional input, normally these
		// values would get pulled from the program environment
		let demo_input = vec![
			"Program Path goes here".to_string(),
			"First Required Input Goes Here".to_string(),
			"Optional Input Here".to_string(),
		];

		let result = app.parse_internal(demo_input.into_iter()).ok().unwrap();

		// This should parse out the first required value, which should be the second item in the
		// demo_input vector
		assert_eq!(
			result.get("first_input").take().unwrap(),
			"First Required Input Goes Here"
		);

		// This should parse out the first optional value, which should be the third item in the
		// demo_input vector
		assert_eq!(
			result.get("optional_second_input").take().unwrap(),
			"Optional Input Here"
		);
	}

	#[test]
	fn test_required_and_flag_parsing() {
		let mut app = generate_test_app();

		// This should result in parsing out the required argument and the provided flag, but not
		// the optional argument
		let demo_input = vec![
			"Program Path goes here".to_string(),
			"Required Arg".to_string(),
			"-vv".to_string(),
		];

		let result = app.parse_internal(demo_input.into_iter()).ok().unwrap();

		assert_eq!(result.get("first_input").take().unwrap(), "Required Arg");

		assert!(!result.contains_key("optional_second_input"));

		assert_eq!(result.get("TestArg").take().unwrap(), "true");
	}

	#[test]
	fn test_default_value_parameters() {
		let mut app = generate_test_app();

		// We expect this to have a result for the default value equal to the set default value in
		// the app spec, in this case "Default Value"
		let demo_input = vec![
			"Program Path Here".to_string(),
			"Required Arg Here".to_string(),
		];

		let result = app.parse_internal(demo_input.into_iter()).ok().unwrap();

		assert_eq!(result.get("HasDefault").take().unwrap(), "Default Value");

		// We expect this one to return the same value as above, but this one is much more likely
		// to break if something goes wrong
		let demo_input = vec![
			"Program Path Here".to_string(),
			"Required Arg Here".to_string(),
			"--default".to_string(),
		];

		let result = app.parse_internal(demo_input.into_iter()).ok().unwrap();

		assert_eq!(result.get("HasDefault").take().unwrap(), "Default Value");

		// This one should have "Alternative Value Here" instead of "Default Value"
		let demo_input = vec![
			"Program Path Here".to_string(),
			"Required Arg Here".to_string(),
			"--default".to_string(),
			"Alternative Value Here".to_string(),
		];

		let result = app.parse_internal(demo_input.into_iter()).ok().unwrap();

		assert_eq!(
			result.get("HasDefault").take().unwrap(),
			"Alternative Value Here"
		);
	}
}

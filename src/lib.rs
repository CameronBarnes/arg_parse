//! This crate is for parsing command line and environment parameters.
//! It's designed to be lightweight and straightforward to use

use crate::Error::{
	DuplicateArgument, MissingArgumentValue, RequiredArgumentMissing, TooManyArguments,
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

#[derive(Debug)]
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

impl Arg {
	fn get_arg_info(&self) -> (String, String, bool, String) {
		let mut str_s = String::new();
		let mut str_l = String::new();
		let mut str_h = String::new();

		if let Some(short) = &self.short {
			str_s.push_str(&collect_strs(short, ","));
		}

		if let Some(long) = &self.long {
			str_l.push_str(&collect_strs(long, ","));
		}

		if let Some(help) = &self.help {
			str_h.push_str(help);
		}

		(str_s, str_l, self.accepts_value, str_h)
	}
}

impl fmt::Display for Arg {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut str = String::new();

		let (short, long, value, help) = self.get_arg_info();

		let mut previous = false;

		if !short.is_empty() {
			str.push_str(&short);
			previous = true;
		}

		if !long.is_empty() {
			if previous {
				str.push('\t');
			}
			str.push_str(&long);
			previous = true;
		}

		if value {
			if previous {
				str.push('\t');
			}
			str.push_str("(accepts value)");
			previous = true;
		}

		if !help.is_empty() {
			if previous {
				str.push('\t');
			}
			str.push_str(&help);
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

		if !self.manual_help_flag
			&& (App::is_manual_help(arg.short.as_ref()) | App::is_manual_help(arg.long.as_ref()))
		{
			self.manual_help_flag = true;
		}

		if !self.manual_version_flag
			&& (App::is_manual_version(arg.short.as_ref())
				| App::is_manual_version(arg.long.as_ref()))
		{
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

		// TODO Ideally I'd like to clean this up eventually, there's more going on in one place
		// here than I'd like, makes it a pain to debug
		while let Some(input) = args.next() {
			let input = input.trim().to_string();
			if let Some(arg) = self.match_arg(&input) {
				if arg.is_completed() {
					return Err(DuplicateArgument(arg.name.clone()));
				}
				// println!("{}", arg.dbg_str());
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
			if self.version.is_some() {
				print!("\t");
			} else {
				print!("  ");
			}
			print!("Author: {author}");
		}

		println!();
		if let Some(about) = &self.about {
			println!("\n{about}\n");
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

		// This next bit needs to be a little bit complicated as we need to first get the maximum
		// string size for each arg so that we can pad the printed strings
		let mut short_min = 0;
		let mut long_min = 0;
		let mut accepts_value = false;

		for arg in &self.args {
			let (short, long, value, _) = arg.get_arg_info();

			if short.len() > short_min {
				short_min = short.len();
			}

			if long.len() > long_min {
				long_min = long.len();
			}

			if value {
				accepts_value = true;
			}
		}

		// Now we get to actually print out the values, making sure to pad them
		for arg in &self.args {
			let (short, long, value, help) = arg.get_arg_info();

			print!("{:short_min$}  ", short);
			print!("{:long_min$}  ", long);
			if value {
				print!("(accepts value)  ");
			} else if accepts_value {
				print!("{:width$}  ", "", width = 15);
			}
			println!("{help}");
		}
	}
}

#[derive(Debug)]
enum Error {
	TooManyArguments(String),
	MissingArgumentValue(String),
	DuplicateArgument(String),
	RequiredArgumentMissing(Vec<String>),
}

#[cfg(test)]
mod tests;

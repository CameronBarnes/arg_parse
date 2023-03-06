use std::collections::HashMap;
use std::{env, fmt, process};
use std::fmt::{Formatter};
use crate::Error::{DuplicateArgument, MissingArgumentValue, TooManyArguments, TooFewArguments, RequiredArgumentMissing};

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
			str.push_str(&collect_strs(short, ","));
			str.push('\n');
		}

		if let Some(long) = &self.long {
			str.push_str(&collect_strs(long, ","));
			str.push('\n');
		}

		if let Some(help) = &self.help {
			str.push_str(help);
		}

		write!(f, "{}", &str)
	}
}

impl Arg {

	pub fn new(name: String) -> Arg {
		Arg {name, short: None, long: None, environment: None, accepts_value: false, required: false, completed: false, help: None, default: None}
	}

	pub fn add_short(&mut self, short: String) -> &mut Arg {

		if let Some(shorts) = &mut self.short {
			shorts.push(short);
		} else {
			self.short = Some(vec!(short));
		}

		self

	}

	pub fn add_long(&mut self, long: String) -> &mut Arg {

		if let Some(longs) = &mut self.long {
			longs.push(long);
		} else {
			self.long = Some(vec!(long));
		}

		self

	}

	pub fn accepts_value(&mut self) -> &mut Arg {

		self.accepts_value = true;

		self

	}

	pub fn help(&mut self, help: String) -> &mut Arg {

		self.help = Some(help);

		self

	}

	pub fn set_default(&mut self, default: String) -> &mut Arg {

		self.default = Some(default);

		self

	}

	pub fn environment(&mut self, env_var: String) -> &mut Arg {

		self.environment = Some(env_var);

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

	name: String,
	version: Option<String>,
	author: Option<String>,
	about: Option<String>,
	args: Vec<Arg>,
	untagged_args_req: i32,
	untagged_args_opt: i32,

}

impl App {

	pub fn new(name: String) -> App {

		App {name, version: None, author: None, about: None, args: Vec::new(), untagged_args_req: 0, untagged_args_opt: 0}

	}

	pub fn version(&mut self, version: String) -> &mut App {

		self.version = Some(version);

		self

	}

	pub fn author(&mut self, author: String) -> &mut App {

		self.author = Some(author);

		self

	}

	pub fn about(&mut self, about: String) -> &mut App {

		self.about = Some(about);

		self

	}

	pub fn arg(&mut self, arg: Arg) -> &mut App {

		self.args.push(arg);

		self

	}

	pub fn set_num_untagged_args_req(&mut self, num: i32) -> &mut App {

		self.untagged_args_req = num;

		self

	}

	pub fn set_num_untagged_args_opt(&mut self, num: i32) -> &mut App {

		self.untagged_args_opt = num;

		self

	}

	fn get_arg(&mut self, name: &str) -> Option<&mut Arg> {

		self.args.iter_mut().find(|arg| arg.name.eq(name))

	}

	fn args_completed(&self) -> bool {
		self.args.iter().all(|arg| arg.is_completed())
	}

	fn args_done(&self) -> bool {
		self.args.iter().all(|arg| arg.is_done())
	}

	fn clear_args(&mut self) {
		for arg in self.args.iter_mut() {
			arg.completed = false;
		}
	}

	fn parse_internal(&mut self, mut args: impl Iterator<Item = String>) -> Result<HashMap<String, Option<String>>, Error> {

		let mut output: HashMap<String, Option<String>> = HashMap::new();

		let mut untagged_req : i32 = -1;
		let mut untagged_opt : i32 = 0;

		// We're always going to put the path of the executable in here, just in case
		output.insert(untagged_req.to_string(), args.next());
		untagged_req += 1;

		while let Some(input) = args.next() {

			if let Some(arg) = self.get_arg(&input) {

				if arg.matches(&input) {

					if arg.is_completed() {
						return Err(DuplicateArgument(arg.name.clone()));
					} else if arg.accepts_value {
						if let Some(value) = args.next() {
							output.insert(arg.name.clone(), Some(value));
						} else {
							return Err(MissingArgumentValue(arg.name.clone()));
						}
					} else {
						output.insert(arg.name.clone(), None);
					}

					arg.set_completed();

				}

			} else if untagged_req < self.untagged_args_req {

				output.insert((untagged_req + untagged_opt).to_string(), Some(input));
				untagged_req += 1;

			} else if untagged_opt < self.untagged_args_opt {

				output.insert((untagged_req + untagged_opt).to_string(), Some(input));
				untagged_opt += 1;

			} else {

				return Err(TooManyArguments(input));

			}

		}

		if untagged_req < self.untagged_args_req {
			return Err(TooFewArguments);
		}

		for arg in self.args.iter_mut().filter(|arg| !arg.completed && arg.environment.is_some()) {

			if env::var(arg.environment.as_ref().expect("Checked during the iter filter")).is_ok() {

			}

		}

		let required_missing: Vec<&Arg> = self.args.iter().filter(|arg| arg.required && !arg.is_done()).collect();

		if !required_missing.is_empty() {
			return Err(RequiredArgumentMissing(required_missing.iter().map(|arg| arg.name.clone()).collect()));
		}

		// TODO fill default values

		self.clear_args();

		Ok(output)

	}

	pub fn parse(&mut self, args: impl Iterator<Item = String>) -> HashMap<String, Option<String>> {

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
						eprintln!("{}\n", self.get_arg(&name).expect("This value should come from a valid arg"));
						self.print_help();
						process::exit(1);
					}
					DuplicateArgument(name) => {
						eprintln!("User provided the following argument twice: ");
						eprintln!("{}\n", self.get_arg(&name).expect("This value should come from a valid arg"));
						self.print_help();
						process::exit(1);
					}
					RequiredArgumentMissing(names) => {
						eprintln!("User did not provide one or more of the following required arguments: ");
						for name in names {
							eprintln!("{}", self.get_arg(&name).expect("This value should come from a valid arg"));
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



	}

}

enum Error {
	TooFewArguments,
	TooManyArguments(String),
	MissingArgumentValue(String),
	DuplicateArgument(String),
	RequiredArgumentMissing(Vec<String>),
}

#[cfg(test)]
mod tests {



}

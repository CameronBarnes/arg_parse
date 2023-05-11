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
#[ignore = "For Manual Testing Only"]
fn test_help() {
	generate_test_app().print_help();
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

#[test]
fn test_fail_duplicate_params() {
	let mut app = generate_test_app();

	// We expect this to result in a DuplicateArgument error
	let demo_input = vec![
		"Program Path Here".to_string(),
		"Required Arg Here".to_string(),
		"-1".to_string(),
		"-one".to_string(),
	];

	let result = app.parse_internal(demo_input.into_iter());

	assert!(matches!(
		result.err().unwrap(),
		ArgumentError::DuplicateArgumentError { .. }
	));
}

use ca1d::{CA, Output, Border, automate};

// end results taken from:
// https://mathworld.wolfram.com/Rule30.html
// https://mathworld.wolfram.com/Rule90.html

#[test]
fn rule30() {
	let ca = CA::new(
		&String::from("1"),
		3,
		2,
		&String::from("30"),
		Border::Ring,
		31,
		false,
	);

	let (_, config) = automate(Output::Null, 0, 15, &ca);

	assert!(config == vec![1,1,0,1,1,1,1,0,0,1,1,0,1,0,0,1,0,1,1,1,1,1,0,0,1,1,1,1,1,1,1]);
}
#[test]
fn rule90() {
	let ca = CA::new(
		&String::from("1"),
		3,
		2,
		&String::from("90"),
		Border::Ring,
		5,
		false,
	);

	let (_, config) = automate(Output::Null, 0, 1, &ca);

	assert!(config == vec![0,1,0,1,0]);
}
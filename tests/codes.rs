use ca1d::{CA, Output, Border, automate};

// end results taken from:
// https://mathworld.wolfram.com/TotalisticCellularAutomaton.html

#[test]
fn code600() {
	let ca = CA::new(
		vec![0,0,1,0,0],
		3,
		3,
		&String::from("600"),
		Border::Ring,
		true,
	);

	let (_, config) = automate(Output::Cell, 0, 3, &ca);

	assert!(config == vec![2,2,0,2,2]);
}
#[test]
fn code777() {
	let ca = CA::new(
		vec![0,0,0,1,0,0,0],
		3,
		3,
		&String::from("777"),
		Border::Ring,
		true,
	);

	let (_, config) = automate(Output::Cell, 0, 3, &ca);

	assert!(config == vec![1,1,0,0,0,1,1]);
}

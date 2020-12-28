use ca1d::{CA, Output, Border, automate, CAEvalType};

// end results taken from:
// https://mathworld.wolfram.com/TotalisticCellularAutomaton.html

#[test]
fn code600() {
    let start_config = vec![0,0,1,0,0];
	let ca = CA::new(
		3,
		3,
		CAEvalType::new(&String::from("code=600")).unwrap(),
		Border::Ring,
	);

	let (_, config) = automate(Output::Cell, 0, 3, &ca, &start_config);

	assert!(config == vec![2,2,0,2,2]);
}

#[test]
fn code777() {
    let start_config = vec![0,0,0,1,0,0,0];
	let ca = CA::new(
		3,
		3,
		CAEvalType::new(&String::from("code=777")).unwrap(),
		Border::Ring,
	);

	let (_, config) = automate(Output::Cell, 0, 3, &ca, &start_config);

	assert!(config == vec![1,1,0,0,0,1,1]);
}

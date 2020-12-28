use ca1d::{CA, Output, Border, automate, CAEvalType};

// end results taken from:
// https://mathworld.wolfram.com/Rule30.html
// https://mathworld.wolfram.com/Rule90.html

#[test]
fn rule30() {
    let start_config = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
	let ca = CA::new(
		3,
		2,
		CAEvalType::new(&String::from("30")).unwrap(),
		Border::Ring,
	);

	let (_, config) = automate(Output::Null,
                               0,
                               15,
                               &ca,
                               &start_config,
                               );

	assert!(config == vec![1,1,0,1,1,1,1,0,0,1,1,0,1,0,0,1,0,1,1,1,1,1,0,0,1,1,1,1,1,1,1]);
}

#[test]
fn rule90() {
    let start_config = vec![0,0,1,0,0];
	let ca = CA::new(
		3,
		2,
		CAEvalType::new(&String::from("rule=90")).unwrap(),
		Border::Ring,
	);

	let (_, config) = automate(Output::Null,
                               0,
                               1,
                               &ca,
                               &start_config,
                               );

	assert!(config == vec![0,1,0,1,0]);
}

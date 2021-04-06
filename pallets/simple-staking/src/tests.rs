use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
	
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
	});
}

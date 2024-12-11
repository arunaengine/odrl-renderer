use anyhow::Result;
use junit_report::{Duration, ReportBuilder, TestCase, TestCaseBuilder, TestSuiteBuilder};

pub fn validate_odrl(raw_json_value: serde_json::Value) -> Result<Vec<u8>> {
    // Create a successful test case
    let test_success = TestCaseBuilder::success("valid odrl", Duration::nanoseconds(1)).build();

    // Create a test case that failed because of a test failure
    let test_failure = TestCase::failure(
        "valid odrl",
        Duration::nanoseconds(1),
        "assert_eq",
        "not equal",
    );

    // Then we create a second test suite called "ts2" and set an explicit time stamp
    // then we add all the test cases from above
    let ts = TestSuiteBuilder::new("odrl validation")
        .add_testcase(test_success)
        .add_testcase(test_failure)
        .build();

    // Last we create a report and add all test suites to it
    let r = ReportBuilder::new().add_testsuite(ts).build();

    // The report can than be written in XML format to any writer
    let mut bytes: Vec<u8> = vec![];
    r.write_xml(&mut bytes)?;

    Ok(bytes)
}

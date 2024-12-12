use anyhow::Result;
use junit_report::{Duration, ReportBuilder, TestCase, TestCaseBuilder, TestSuiteBuilder};

use crate::template::{get_string_from_asset, get_string_from_party};

pub fn validate_odrl(raw_json_value: serde_json::Value) -> Result<Vec<u8>> {
    // Create a successful test case
    let mut ts = TestSuiteBuilder::new("odrl validation");

    let parsed = match serde_json::from_value::<generic_odrl::policy::GenericPolicy>(raw_json_value)
    {
        Ok(p) => {
            ts.add_testcase(
                TestCaseBuilder::success("valid odrl", Duration::nanoseconds(1)).build(),
            );
            Some(p)
        }
        Err(e) => {
            let test_case = TestCase::failure(
                "invalid odrl",
                Duration::nanoseconds(1),
                "invalid json",
                &e.to_string(),
            );
            ts.add_testcase(test_case);
            None
        }
    };

    if let Some(parsed) = parsed {
        match parsed.policy_type.as_str() {
            "Offer" => {
                if let Some(_) = get_string_from_party(&parsed.assigner) {
                    ts.add_testcase(TestCase::success(
                        "valid assigner",
                        Duration::nanoseconds(1),
                    ));
                } else {
                    ts.add_testcase(TestCase::failure(
                        "invalid assigner",
                        Duration::nanoseconds(1),
                        "assigner set",
                        &format!("assigner is not set"),
                    ));
                }

                if let Some(_) = get_string_from_asset(&parsed.target) {
                    ts.add_testcase(TestCase::success(
                        "valid target asset",
                        Duration::nanoseconds(1),
                    ));
                } else {
                    ts.add_testcase(TestCase::failure(
                        "invalid target",
                        Duration::nanoseconds(1),
                        "target set",
                        &format!("target is not set"),
                    ));
                }
            }
            "Set" => {}
            "Agreement" => {
                if let Some(_) = get_string_from_party(&parsed.assigner) {
                    ts.add_testcase(TestCase::success(
                        "valid assigner",
                        Duration::nanoseconds(1),
                    ));
                } else {
                    ts.add_testcase(TestCase::failure(
                        "invalid assigner",
                        Duration::nanoseconds(1),
                        "assigner set",
                        &format!("assigner is not set"),
                    ));
                }

                if let Some(_) = get_string_from_party(&parsed.assignee) {
                    ts.add_testcase(TestCase::success(
                        "valid assignee",
                        Duration::nanoseconds(1),
                    ));
                } else {
                    ts.add_testcase(TestCase::failure(
                        "invalid assignee",
                        Duration::nanoseconds(1),
                        "assignee set",
                        &format!("assignee is not set"),
                    ));
                }

                if let Some(_) = get_string_from_asset(&parsed.target) {
                    ts.add_testcase(TestCase::success(
                        "valid target asset",
                        Duration::nanoseconds(1),
                    ));
                } else {
                    ts.add_testcase(TestCase::failure(
                        "invalid target",
                        Duration::nanoseconds(1),
                        "target set",
                        &format!("target is not set"),
                    ));
                }
            }
            e => {
                ts.add_testcase(TestCase::failure(
                    "invalid odrl",
                    Duration::nanoseconds(1),
                    "invalid policy type",
                    &format!(
                        "Policy type: {e} is not allowed. Must be one of [Set | Offer | Agreement]"
                    ),
                ));
            }
        }
    }

    // Last we create a report and add all test suites to it
    let r = ReportBuilder::new().add_testsuite(ts.build()).build();

    // The report can than be written in XML format to any writer
    let mut bytes: Vec<u8> = vec![];
    r.write_xml(&mut bytes)?;

    Ok(bytes)
}

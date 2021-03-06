//
// Copyright (C) 2018 Kubos Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use super::*;

#[test]
fn ack_default() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "NONE"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn ack_noop() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let noop = r#"mutation {
            noop {
                success
            }
        }"#;

    service.process(noop.to_owned());

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "NOOP"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn ack_control_power() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let mutation = r#"mutation {
            controlPower(state: RESET) {
                success
            }
        }"#;

    service.process(mutation.to_owned());

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "CONTROL_POWER"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn ack_configure_hardware() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let mutation = r#"mutation {
            configureHardware(config: SECONDARY) {
                success
            }
        }"#;

    service.process(mutation.to_owned());

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "CONFIGURE_HARDWARE"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn ack_arm() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let mutation = r#"mutation {
            arm(state: ARM) {
                success
            }
        }"#;

    service.process(mutation.to_owned());

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "ARM"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn ack_deploy() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let mutation = r#"mutation {
            deploy(time: 5) {
                success
            }
        }"#;

    service.process(mutation.to_owned());

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "DEPLOY"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn ack_test_hardware() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let mutation = r#"mutation {
            testHardware(test: INTEGRATION) {
                ... on IntegrationTestResults {
                    success
                }
            }
        }"#;

    service.process(mutation.to_owned());

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "TEST_HARDWARE"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn ack_issue_raw_command() {
    let mock = mock_new!();

    let service = service_new!(mock);

    let mutation = r#"mutation {
            issueRawCommand(command: "01"){
                success
            }
        }"#;

    service.process(mutation.to_owned());

    let query = r#"{
            ack
        }"#;

    let expected = json!({
            "ack": "ISSUE_RAW_COMMAND"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

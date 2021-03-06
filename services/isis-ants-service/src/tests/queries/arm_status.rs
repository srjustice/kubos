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
fn arm_status_armed() {
    let mock = mock_new!();

    mock.get_deploy.return_value(Ok(DeployStatus {
        sys_armed: true,
        ..Default::default()
    }));

    let service = service_new!(mock);

    let query = r#"
        {
            armStatus
        }"#;

    let expected = json!({
            "armStatus": "ARMED"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

#[test]
fn arm_status_disarmed() {
    let mock = mock_new!();

    mock.get_deploy.return_value(Ok(DeployStatus {
        sys_armed: false,
        ..Default::default()
    }));

    let service = service_new!(mock);

    let query = r#"
        {
            armStatus
        }"#;

    let expected = json!({
            "armStatus": "DISARMED"
    });

    assert_eq!(service.process(query.to_owned()), wrap!(expected));
}

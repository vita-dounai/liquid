// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/01-expand-noop.rs");
    t.pass("tests/ui/pass/02-expand-body.rs");
    t.pass("tests/ui/pass/03-expand-enum.rs");
    t.pass("tests/ui/pass/04-expand-array.rs");
    t.pass("tests/ui/pass/05-expand-tuple.rs");
    t.pass("tests/ui/pass/06-expand-grouped-range.rs");
    t.pass("tests/ui/pass/07-expand-add-one.rs");
    t.compile_fail("tests/ui/fail/01-missing-var.rs");
    t.compile_fail("tests/ui/fail/02-missing-in-keyword.rs");
    t.compile_fail("tests/ui/fail/03-invalid-range.rs");
}

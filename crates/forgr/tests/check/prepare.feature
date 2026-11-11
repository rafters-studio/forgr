# Generated from the legion requirement. Do not hand-edit this file.
Feature: the SQL check prepares what it finds; text that does not parse is unchecked (FR-FORGR-007)

  @criterion-01a1c93d-97d0-7461-aceb-ea3a6efa7aaa
  Scenario: unparsable SQL-looking text is unchecked
    Given a crate that calls schema! over the macro toy's schema.json fixture and whose src holds the literal INSERT OR IGNORE should produce zero affected on re-run
    When the developer runs the owning crate's tests
    Then the check lists that literal as unchecked and does not fail on it

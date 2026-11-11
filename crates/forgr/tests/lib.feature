# Generated from the legion requirement. Do not hand-edit this file.
Feature: schema!, called as a user calls it through forgr (FR-FORGR-001, FR-FORGR-002)

  @criterion-01a1c93d-7ee8-7711-a173-c91cfe78b52b
  Scenario: forgr leaves schema.json as it found it
    Given a crate that calls schema! over the macro toy's schema.json fixture
    When the developer builds and runs the crate's tests
    Then the crate's schema.json is unchanged

  @criterion-01a1c93d-95a0-7d81-9760-54ca8ed0d8e6
  Scenario: each table gets its own module
    Given an owning crate that calls schema! over kessel's schema.json fixture
    When its tests name the generated modules
    Then they can name a combat_styles module and an objects module

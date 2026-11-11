# Generated from the legion requirement. Do not hand-edit this file.
Feature: a table's mock, behind fake (FR-FORGR-005)

  @criterion-01a1c93d-96dc-7463-9d39-dc003474649d
  Scenario: a mock of no rows
    Given the empty test store for the macro toy's schema.json fixture
    When the test inserts notes::mock(0)
    Then insert returns no rows

  @criterion-01a1c93d-96dc-7463-9d39-dbe167018b7a
  Scenario: a mock of a set number of rows
    Given the empty test store for kessel's schema.json fixture
    When the test inserts combat_styles::mock(8)
    Then the store holds 8 combat_styles rows

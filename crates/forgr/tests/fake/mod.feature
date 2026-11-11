# Generated from the legion requirement. Do not hand-edit this file.
Feature: a table's mock, behind fake (FR-FORGR-005)

  @criterion-01a1c93d-96dc-7463-9d39-dc003474649d
  Scenario: a mock of no rows
    Given the empty test store for the macro toy's schema.json fixture
    When the test inserts notes::mock(0)
    Then insert returns no rows

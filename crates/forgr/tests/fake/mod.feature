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

  @criterion-01a1c93d-96dc-7463-9d39-dbff81cd24cd
  Scenario: a mock of a random number of rows
    Given the empty test store for the macro toy's schema.json fixture
    When the test inserts events::mock(3..32)
    Then the store holds at least 3 and at most 31 events rows

  @criterion-01a1c93d-96dc-7463-9d39-dc38a260301f
  Scenario: a fixed override
    Given the empty test store for kessel's schema.json fixture
    When the test inserts combat_styles::mock(8) with attack_type Fixed to force
    Then every combat_styles row has attack_type force

  @criterion-01a1c93d-96dc-7463-9d39-dc423f2a7707
  Scenario: a picked override
    Given the empty test store for the macro toy's schema.json fixture
    When the test inserts events::mock(12) with status Pick from -1, 0, 1 and 2
    Then every events row has a status of -1, 0, 1 or 2

  @criterion-01a1c93d-96dc-7463-9d39-dc5e25a8383f
  Scenario: a per-row override
    Given the empty test store for the macro toy's schema.json fixture
    When the test inserts events::mock(12) with body made per row as kid and the row's number
    Then each events row's body is kid and that row's number

# Generated from the legion requirement. Do not hand-edit this file.
Feature: the empty test store, from the generated module (FR-FORGR-003)

  @criterion-01a1c93d-9615-7a51-b973-de20c903a0e9
  Scenario: the store has the schema's tables
    Given kessel's schema.json fixture
    When a test takes the empty test store
    Then the store has the tables combat_styles and objects

  @criterion-01a1c93d-9615-7a51-b973-de376b6bcf66
  Scenario: the store starts empty
    Given kessel's schema.json fixture
    When a test takes the empty test store
    Then the combat_styles and objects tables have no rows

  @criterion-01a1c93d-9615-7a51-b973-de45e8d50cce
  Scenario: the store enforces foreign keys
    Given the empty test store for kessel's schema.json fixture, where combat_styles has a foreign key to objects
    When the test writes a combat_styles row whose foreign key names no objects row
    Then the store refuses the row

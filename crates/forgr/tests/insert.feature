# Generated from the legion requirement. Do not hand-edit this file.
Feature: insert, from a table's factory or mock (FR-FORGR-004, FR-FORGR-005)

  @criterion-01a1c93d-967c-7370-9edc-17c641cc75fa
  Scenario: an unset column gets the schema's default
    Given the empty test store for kessel's schema.json fixture, where objects.template_guid defaults to empty text
    When the test inserts an objects factory with game_id g1, stable_id s, payload_hash p, guid x, fqn class.pc.advanced.sage, kind class and json {} and no template_guid
    Then the inserted objects row has an empty template_guid

  @criterion-01a1c93d-967c-7370-9edc-17d71f5ef4f4
  Scenario: insert returns the row it wrote
    Given the empty test store for kessel's schema.json fixture
    When the test inserts an objects factory with game_id g1, stable_id s, payload_hash p, guid x, fqn class.pc.advanced.sage, kind class and json {}
    Then insert returns the objects row as written, with game_id g1

  @criterion-01a1c93d-96dc-7463-9d39-dc119a55eac4
  Scenario: a mock's insert returns its rows
    Given the empty test store for kessel's schema.json fixture
    When the test inserts combat_styles::mock(8)
    Then insert returns 8 combat_styles rows, each with its combat_style_id

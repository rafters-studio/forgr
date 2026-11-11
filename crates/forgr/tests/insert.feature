# Generated from the legion requirement. Do not hand-edit this file.
Feature: insert, from a table's factory (FR-FORGR-004)

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

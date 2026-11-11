# Generated from the legion requirement. Do not hand-edit this file.
Feature: forgr reads schema.json in migratr's format, whoever wrote it, and never writes it (FR-FORGR-001)

  @criterion-01a1c9a7-387f-7230-977a-5c3536e524ca
  Scenario: the model leaves out virtual and shadow tables
    Given the writer schema.json fixture, which holds the FTS5 table notes_fts and its shadow tables
    When forgr-schema reads it into the schema model
    Then the model's tables include no notes_fts and no table named notes_fts_ followed by anything

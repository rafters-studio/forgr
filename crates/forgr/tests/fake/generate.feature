# Generated from the legion requirement. Do not hand-edit this file.
Feature: a mock's values, behind fake (FR-FORGR-005)

  @criterion-01a1c93d-96dc-7463-9d39-dc6c90d5a909
  Scenario: a TEXT primary key gets a UUIDv7
    Given the empty test store for the macro toy's schema.json fixture, where the primary key of notes is TEXT
    When the test inserts notes::mock(8)
    Then every notes row's id is a UUIDv7

  @criterion-01a1c950-1f37-7ed0-a2af-6aa3cb446844
  Scenario: an INTEGER primary key gets a positive integer
    Given the empty test store for the macro toy's schema.json fixture, where the primary key of events is INTEGER
    When the test inserts events::mock(8)
    Then every events row's id is a positive integer

  @criterion-01a1c93d-96dc-7463-9d39-dc9003e54f0b
  Scenario: a nullable column is null in some rows
    Given the empty test store for the macro toy's schema.json fixture, where notes.body is nullable and not a foreign key
    When the test inserts notes::mock(50)
    Then some notes rows have body null

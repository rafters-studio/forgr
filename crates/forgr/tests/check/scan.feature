# Generated from the legion requirement. Do not hand-edit this file.
Feature: the SQL check scans src, not tests/ (FR-FORGR-007)

  @criterion-01a1c93d-97d0-7461-aceb-ea58e8189cfa
  Scenario: tests/ is not checked
    Given a crate that calls schema! over the macro toy's schema.json fixture and whose tests/ holds a query naming the column author of notes, which the fixture lacks
    When the developer runs the owning crate's tests
    Then the check does not fail on that query

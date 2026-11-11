# Generated from the legion requirement. Do not hand-edit this file.
Feature: schema!, called as a user calls it through forgr (FR-FORGR-001, FR-FORGR-002, FR-FORGR-004, FR-FORGR-006, FR-FORGR-008, FR-FORGR-009)

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

  @criterion-01a1c93d-7ee8-7711-a173-c92ee772b411
  Scenario: forgr needs no database file
    Given a crate that calls schema! over the macro toy's schema.json fixture and has no database file
    When a test inserts a notes factory with id n1, owner ana, updated_at t and kind memo into the empty test store
    Then insert returns the notes row with owner ana

  @criterion-01a1c93d-967c-7370-9edc-177c0288cbf5
  Scenario: a factory sets columns by name
    Given the empty test store for kessel's schema.json fixture, holding an objects row with game_id g1, stable_id s, payload_hash p, guid x, fqn class.pc.advanced.sage, kind class and json {}
    When the test inserts a combat_styles factory setting combat_style_id g1, fqn class.pc.advanced.sage, fqn_segment force_wizard, display_segment sage, faction republic and attack_type force
    Then the store holds a combat_styles row with those values

  @criterion-01a1c93d-967c-7370-9edc-1785b8cb3836
  Scenario: a keyword column gets a raw identifier
    Given the empty test store for the edge schema.json fixture, where kinds has a TEXT NOT NULL column named type
    When the test inserts a kinds factory setting id and setting type with r#type
    Then the test compiles and the inserted kinds row carries the type it was given

  @criterion-01a1c93d-967c-7370-9edc-17b9705d5d76
  Scenario: a factory requires every primary-key column
    Given kessel's schema.json fixture, where the primary key of combat_styles is combat_style_id
    When the test inserts a combat_styles factory with only fqn set
    Then the build fails with an error naming combat_styles::col::combat_style_id

  @criterion-01a1c93d-967c-7370-9edc-17eaf2e9f952
  Scenario: a setter's type follows the column's SQLite type
    Given the empty test store for the macro toy's schema.json fixture, where events.status is INTEGER and events.body is TEXT
    When the test inserts an events factory with id 1, ty 3, status 0 and body root
    Then the test compiles and the inserted row carries status 0 and body root

  @criterion-01a1c950-1faa-7a53-a1ae-33e87e8f40fe
  Scenario: a nullable foreign key left Fake is NULL
    Given the empty test store for the macro toy's schema.json fixture, where events.source is a nullable foreign key
    When the test inserts events::mock(8) with no override for source
    Then every events row's source is NULL

  @criterion-01a1c93d-9750-70f0-959d-b47c82bfe611
  Scenario: a mock builds parents for a composite foreign key
    Given the empty test store for the macro toy's schema.json fixture, where note_tags has the required composite foreign key (owner, name) to tags
    When the test inserts note_tags::mock(8)
    Then the store holds 8 tags rows and no foreign-key violations

  @criterion-01a1c93d-9750-70f0-959d-b4594d9ef0c6
  Scenario: a mock attaches its rows to one parent
    Given the empty test store for the macro toy's schema.json fixture, holding an events row inserted by a factory with id 1, ty 3, status 0 and body root
    When the test inserts events::mock(12) with source Fixed to 1
    Then every new events row's source is 1

  @criterion-01a1c950-1faa-7a53-a1ae-33fbf772728e
  Scenario: a required self-reference is set with Fixed
    Given the empty test store for the edge schema.json fixture, holding one nodes row inserted by a factory whose parent_id is its own id
    When the test inserts nodes::mock(8) with parent_id Fixed to that row's id
    Then every new nodes row's parent_id is that row's id

  @criterion-01a1c93d-9845-7a72-8e5d-bbcfea70927f
  Scenario: an events table has a factory
    Given the empty test store for the macro toy's schema.json fixture
    When the test inserts an events factory with id 1, ty 3, status 0 and body root
    Then the store holds that events row

  @criterion-01a1c93d-98c8-7142-85fe-29dd70a3aa23
  Scenario: an FTS5 table has no factory
    Given the macro toy's schema.json fixture, which holds the FTS5 table notes_fts
    When a test calls notes_fts::factory()
    Then the test fails to compile

  @criterion-01a1c93d-98c8-7142-85fe-29e4537183d6
  Scenario: an FTS5 table has no mock
    Given the macro toy's schema.json fixture, which holds the FTS5 table notes_fts, and forgr's fake feature on
    When a test calls notes_fts::mock(8)
    Then the test fails to compile

  @criterion-01a1c96f-deab-7fc2-bbe4-84d31c107a34
  Scenario: the SQL check needs no database file
    Given a crate that calls schema! over the macro toy's schema.json fixture and has no database file
    When the developer runs the crate's tests
    Then the emitted SQL check runs and reports its checked, failed and unchecked counts

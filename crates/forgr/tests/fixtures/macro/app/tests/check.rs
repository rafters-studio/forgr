//! The macro toy's C2 case (RESEARCH 01a1c917-ba6c-7553-bffe-c648a5f16f44),
//! written for forgr #10: "a test binary holding only schema!()", over the
//! toy's schema.json one directory up. Its emitted forgr_sql_matches_schema
//! runs the check over this crate's src; M7 and C3 read its report.
forgr::schema!("../schema.json");

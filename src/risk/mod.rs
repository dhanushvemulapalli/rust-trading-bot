//! Risk management — sits between strategy signals and order execution.
//!
//! The risk manager must approve every signal before an order is submitted.
//! A raw strategy signal MUST NOT directly create an order.

pub mod manager;

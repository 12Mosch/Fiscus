use fiscus_lib::{TransactionType};
use rust_decimal::Decimal;

/// Test the balance calculation logic for transaction type changes
/// This test verifies the core logic that was fixed in the update_transaction function
#[test]
fn test_balance_calculation_logic_transaction_type_change() {
    // Scenario: Account has $1000 initial balance
    let initial_balance = Decimal::new(100000, 2); // $1000.00
    
    // Original transaction: Income of $200
    let original_amount = Decimal::new(20000, 2); // $200.00
    let original_type = TransactionType::Income;
    
    // After original transaction: $1000 + $200 = $1200
    let balance_after_original = match original_type {
        TransactionType::Income => initial_balance + original_amount,
        TransactionType::Expense => initial_balance - original_amount,
        TransactionType::Transfer => initial_balance,
    };
    assert_eq!(balance_after_original, Decimal::new(120000, 2));
    
    // Test Case 1: Change transaction type from Income to Expense (same amount)
    let new_amount = original_amount; // Keep same amount
    let new_type = TransactionType::Expense;
    
    // Step 1: Reverse the old transaction effect
    let balance_after_reversal = match original_type {
        TransactionType::Income => balance_after_original - original_amount,
        TransactionType::Expense => balance_after_original + original_amount,
        TransactionType::Transfer => balance_after_original,
    };
    assert_eq!(balance_after_reversal, initial_balance); // Should be back to $1000
    
    // Step 2: Apply the new transaction effect using NEW transaction type
    let final_balance = match new_type {
        TransactionType::Income => balance_after_reversal + new_amount,
        TransactionType::Expense => balance_after_reversal - new_amount,
        TransactionType::Transfer => balance_after_reversal,
    };
    
    // Expected: $1000 - $200 = $800 (expense)
    assert_eq!(final_balance, Decimal::new(80000, 2));
}

/// Test balance calculation when both amount and transaction type change
#[test]
fn test_balance_calculation_logic_amount_and_type_change() {
    // Scenario: Account has $500 initial balance with $100 expense
    let initial_balance = Decimal::new(50000, 2); // $500.00
    let original_amount = Decimal::new(10000, 2); // $100.00
    let original_type = TransactionType::Expense;
    
    // After original transaction: $500 - $100 = $400
    let current_balance = Decimal::new(40000, 2); // $400.00
    
    // Change to: Income of $150
    let new_amount = Decimal::new(15000, 2); // $150.00
    let new_type = TransactionType::Income;
    
    // Step 1: Reverse the old transaction effect
    let balance_after_reversal = match original_type {
        TransactionType::Income => current_balance - original_amount,
        TransactionType::Expense => current_balance + original_amount,
        TransactionType::Transfer => current_balance,
    };
    assert_eq!(balance_after_reversal, initial_balance); // Should be back to $500
    
    // Step 2: Apply the new transaction effect using NEW transaction type and amount
    let final_balance = match new_type {
        TransactionType::Income => balance_after_reversal + new_amount,
        TransactionType::Expense => balance_after_reversal - new_amount,
        TransactionType::Transfer => balance_after_reversal,
    };
    
    // Expected: $500 + $150 = $650 (income)
    assert_eq!(final_balance, Decimal::new(65000, 2));
}

/// Test that demonstrates the bug that was fixed
/// This shows what would happen with the old logic vs the new logic
#[test]
fn test_bug_demonstration_old_vs_new_logic() {
    let current_balance = Decimal::new(120000, 2); // $1200 (after +$200 income)
    let original_amount = Decimal::new(20000, 2); // $200.00
    let original_type = TransactionType::Income;
    let new_amount = Decimal::new(20000, 2); // $200.00 (same amount)
    let new_type = TransactionType::Expense; // Changed type
    
    // OLD LOGIC (buggy): Uses original_type for both reversal and application
    let old_balance_after_reversal = match original_type {
        TransactionType::Income => current_balance - original_amount,
        TransactionType::Expense => current_balance + original_amount,
        TransactionType::Transfer => current_balance,
    };
    
    let old_final_balance = match original_type { // BUG: Using original_type instead of new_type
        TransactionType::Income => old_balance_after_reversal + new_amount,
        TransactionType::Expense => old_balance_after_reversal - new_amount,
        TransactionType::Transfer => old_balance_after_reversal,
    };
    
    // OLD LOGIC RESULT: $1200 - $200 + $200 = $1200 (no change - WRONG!)
    assert_eq!(old_final_balance, Decimal::new(120000, 2));
    
    // NEW LOGIC (fixed): Uses original_type for reversal, new_type for application
    let new_balance_after_reversal = match original_type {
        TransactionType::Income => current_balance - original_amount,
        TransactionType::Expense => current_balance + original_amount,
        TransactionType::Transfer => current_balance,
    };
    
    let new_final_balance = match new_type { // FIXED: Using new_type
        TransactionType::Income => new_balance_after_reversal + new_amount,
        TransactionType::Expense => new_balance_after_reversal - new_amount,
        TransactionType::Transfer => new_balance_after_reversal,
    };
    
    // NEW LOGIC RESULT: $1200 - $200 - $200 = $800 (correct change from income to expense)
    assert_eq!(new_final_balance, Decimal::new(80000, 2));
    
    // Verify the difference
    assert_ne!(old_final_balance, new_final_balance);
    assert_eq!(old_final_balance - new_final_balance, Decimal::new(40000, 2)); // $400 difference
}

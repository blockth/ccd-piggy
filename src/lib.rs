use concordium_std::{
    test_infrastructure::{TestHost, TestInitContext, TestReceiveContext, TestStateBuilder},
    *,
};

//everyone should be able to put money
// but only the owner can smash it and retrieve the CCD
// once its smashed it's not possible to add CCD

// create an enum to keep the state of the piggy bank
// smashed or not

// store on chain so need Serialize trait occurs
// Clone and Copy for copying
// Debug for printing
// Eq and PartialEq for comparison of state
#[derive(Serialize, PartialEq, Eq, Debug, Clone, Copy)]
enum PiggyBankState {
    Intact,
    Smashed,
}

/// have to implement an init function for a valid contract
/// when called init() it creates an instance
#[init(contract = "PiggyBank")]
fn piggy_init<S: HasStateApi>(
    _ctx: &impl HasInitContext,           //get state info argument trait
    _state_builder: &mut StateBuilder<S>, // set, map, box create
) -> InitResult<PiggyBankState> {
    //return Result<PiggyBankState,Reject>
    Ok(PiggyBankState::Intact)
}

// #[receive(contract = "MyContract", name = "some_interaction")]
// fn some_receive<S: HasStateApi>(
//     ctx: &impl HasReceiveContext,
//     host: &impl HasHost<MyState, StateApiType = S>,
// ) -> ReceiveResult<MyReturnValue> {
//     todo!()
// }

// should be payable function
// payable attribute makes sure retrieve the ccd transferred again
#[receive(contract = "PiggyBank", name = "insert", payable)]
fn piggy_insert<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<PiggyBankState, StateApiType = S>,
    _amount: Amount,
) -> ReceiveResult<()> {
    ensure!(*host.state() == PiggyBankState::Intact);
    Ok(())
}
// state is immutable default, add mutable attribute
#[receive(contract = "PiggyBank", name = "smash", mutable)]
fn piggy_smash<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<PiggyBankState, StateApiType = S>,
) -> ReceiveResult<()> {
    let owner = ctx.owner(); // this returns the account address of the contract instance owner
    let sender = ctx.sender(); // who triggered this function? this could be both an Account address or or Contract instance address

    ensure!(sender.matches_account(&owner));
    ensure!(*host.state() == PiggyBankState::Intact);

    *host.state_mut() = PiggyBankState::Smashed;
    let balance = host.self_balance(); // get contract balance
    Ok(host.invoke_transfer(&owner, balance)?)
}

// view state
#[receive(contract = "PiggyBank", name = "view")]
fn piggy_view<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<PiggyBankState, StateApiType = S>,
) -> ReceiveResult<(PiggyBankState, Amount)> {
    let current_state = *host.state();
    let current_balance = host.self_balance();
    Ok((current_state, current_balance))
}

// Piggy test

#[cfg(test)]
mod tests {
    use super::*;
    use test_infrastructure::*; // concordium-std includes templates/stubs
}

#[test]
fn test_init() {
    let ctx = TestInitContext::empty();
    let mut state_builder = TestStateBuilder::new();
    let state_result = piggy_init(&ctx, &mut state_builder);
    let state = state_result.expect("Contract initialization error");

    assert_eq!(state, PiggyBankState::Intact, "Initially have to be intact");
}

#[test]
fn test_insert() {
    let ctx = TestReceiveContext::empty();
    let mut host = TestHost::new(PiggyBankState::Intact, TestStateBuilder::new());

    let amount = Amount::from_micro_ccd(100);

    let result = piggy_insert(&ctx, &host, amount);
    host.set_self_balance(amount);
    assert!(result.is_ok(), "Insert CCD error");

    assert_eq!(
        *host.state(),
        PiggyBankState::Intact,
        "Piggy bank state should be intact"
    );

    assert_eq!(host.self_balance(), amount);
}

#[test]
fn test_smash() {
    let mut ctx = TestReceiveContext::empty();
    let owner = AccountAddress([0u8; 32]); // concordium addresses are 32 bytes

    ctx.set_owner(owner);
    let sender = Address::Account(owner);
    ctx.set_sender(sender);

    let mut host = TestHost::new(PiggyBankState::Intact, TestStateBuilder::new());
    let balance = Amount::from_micro_ccd(100);
    host.set_self_balance(balance);

    let result = piggy_smash(&ctx, &mut host);
    assert!(result.is_ok(), "smash failed!");
    assert_eq!(*host.state(), PiggyBankState::Smashed, "cant smash");

    assert_eq!(
        host.get_transfers(),
        [(owner, balance)],
        "Error in transfer"
    );
}

#[test]
fn view_test() {
    let ctx = TestReceiveContext::empty();
    let state_builder = TestStateBuilder::new();
    let host = TestHost::new(PiggyBankState::Intact, state_builder);

    let amount = Amount::from_micro_ccd(100);

    let owner = AccountAddress([0u8; 32]); // concordium addresses are 32 bytes

    let result = piggy_insert(&ctx, &host, amount);
    println!("{:?}", host.self_balance());
    // let balance = Amount::from_micro_ccd(100);
    // assert_eq!(
    //     host.get_transfers(),
    //     [(owner, balance)],
    //     "Error in transfer"
    // );

    // println!("{:?}", host.get_transfers());

    // let res_view = piggy_view(&ctx, &host);
    // assert!(res_view.is_ok(), "failed to view");

    // assert_eq!(*host.state(), PiggyBankState::Intact, "state error");

    //assert_eq!(host.self_balance(), test_amount, "amount error");
}

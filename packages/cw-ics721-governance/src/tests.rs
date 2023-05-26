use cosmwasm_std::testing::{mock_dependencies, mock_info};

use super::*;

fn mock_addresses() -> [Addr; 4] {
    [
        Addr::unchecked("owner"),
        Addr::unchecked("other"),
        Addr::unchecked("origin"),
        Addr::unchecked("ark"),
    ]
}

//------------------------------------------------------------------------------
// Unit Tests
//------------------------------------------------------------------------------

#[test]
fn instantiate_origin_specified() {
    let mut deps = mock_dependencies();
    let [owner, _, origin, foo] = mock_addresses();
    let transfer_fee = Some(coin(100, "uark"));
    let info = mock_info(foo.as_str(), &vec![]);
    instantiate(
        deps.as_mut(),
        info,
        Some(owner.to_string()),
        Some(origin.to_string()),
        transfer_fee.clone(),
    )
    .unwrap();

    // governance returned is same as governance stored.
    assert_eq!(
        GOVERNANCE.load(deps.as_ref().storage).unwrap(),
        Governance {
            owner: Some(owner),
            origin,
            transfer_fee,
        },
    );
}

#[test]
fn instantiate_sender_is_origin() {
    let mut deps = mock_dependencies();
    let [owner, _, _, foo] = mock_addresses();
    let transfer_fee = Some(coin(100, "uark"));
    let info = mock_info(foo.as_str(), &vec![]);
    instantiate(
        deps.as_mut(),
        info,
        Some(owner.to_string()),
        None,
        transfer_fee.clone(),
    )
    .unwrap();

    // governance returned is same as governance stored.
    assert_eq!(
        GOVERNANCE.load(deps.as_ref().storage).unwrap(),
        Governance {
            owner: Some(owner),
            origin: foo,
            transfer_fee,
        },
    );
}

#[test]
fn instantiate_governance_no_owner() {
    let mut deps = mock_dependencies();
    let [_, _, origin, foo] = mock_addresses();

    let info = mock_info(foo.as_str(), &vec![]);
    instantiate(deps.as_mut(), info, None, Some(origin.to_string()), None).unwrap();
    assert_eq!(
        GOVERNANCE.load(deps.as_ref().storage).unwrap(),
        Governance {
            owner: None,
            origin,
            transfer_fee: None,
        },
    );
}

#[test]
fn asserting_owner() {
    let mut deps = mock_dependencies();
    let [owner, other, origin, foo] = mock_addresses();

    // case 1. owner is set
    {
        let info = mock_info(foo.as_str(), &vec![]);
        instantiate(
            deps.as_mut(),
            info,
            Some(owner.to_string()),
            Some(origin.to_string()),
            None,
        )
        .unwrap();

        // sender is owner
        let res = assert_owner(deps.as_ref().storage, &owner);
        assert!(res.is_ok());
        // sender is not owner
        let res = assert_owner(deps.as_ref().storage, &other);
        assert_eq!(
            res.unwrap_err(),
            GovernanceError::NotOwner(other.to_string())
        );
    }

    // case 2. owner is not set
    {
        update_owner(deps.as_mut().storage, None).unwrap();

        let res = assert_owner(deps.as_ref().storage, &owner);
        assert_eq!(res.unwrap_err(), GovernanceError::NoOwner);
    }
}

#[test]
fn executing_owner() {
    let mut deps = mock_dependencies();
    let [owner, other, origin, foo] = mock_addresses();

    // case 1. owner is set
    {
        let info = mock_info(foo.as_str(), &vec![]);
        instantiate(
            deps.as_mut(),
            info,
            Some(owner.to_string()),
            Some(origin.to_string()),
            None,
        )
        .unwrap();

        // sender is owner
        let res = execute_owner(deps.as_mut(), &owner, other.to_string());
        assert!(res.is_ok());
        // sender is not owner
        let res = execute_owner(deps.as_mut(), &foo, owner.to_string());
        assert_eq!(res.unwrap_err(), GovernanceError::NotOwner(foo.to_string()));
    }

    // case 2. owner is not set
    {
        // first set owner to none
        update_owner(deps.as_mut().storage, None).unwrap();
        // now call againclone
        let res = execute_owner(deps.as_mut(), &other, "foo".to_string());
        assert_eq!(res.unwrap_err(), GovernanceError::NoOwner);
    }
}

#[test]
fn executing_origin() {
    let mut deps = mock_dependencies();
    let [owner, other, origin, foo] = mock_addresses();

    // case 1. owner is set
    {
        let info = mock_info(foo.as_str(), &vec![]);
        instantiate(
            deps.as_mut(),
            info,
            Some(owner.to_string()),
            Some(origin.to_string()),
            None,
        )
        .unwrap();

        // sender is owner
        let res = execute_origin(deps.as_mut(), &owner, other.to_string());
        assert!(res.is_ok());
        // sender is not owner
        let res = execute_origin(deps.as_mut(), &other, other.to_string());
        assert_eq!(
            res.unwrap_err(),
            GovernanceError::NotOwner(other.to_string())
        );
    }

    // case 2. owner is not set
    {
        // first set owner to none
        update_owner(deps.as_mut().storage, None).unwrap();

        let res = execute_origin(deps.as_mut(), &other, origin.to_string());
        assert_eq!(res.unwrap_err(), GovernanceError::NoOwner);
    }
}

#[test]
fn executing_transfer_fee() {
    let mut deps = mock_dependencies();
    let [owner, other, origin, foo] = mock_addresses();
    let transfer_fee = Some(coin(100, "uark"));

    // case 1. owner is set
    {
        let info = mock_info(foo.as_str(), &vec![]);
        instantiate(
            deps.as_mut(),
            info,
            Some(owner.to_string()),
            Some(origin.to_string()),
            None,
        )
        .unwrap();

        // sender is owner
        let res = execute_transfer_fee(deps.as_mut(), &owner, transfer_fee.clone());
        assert!(res.is_ok());
        // sender is not owner
        let res = execute_transfer_fee(deps.as_mut(), &other, transfer_fee.clone());
        assert_eq!(
            res.unwrap_err(),
            GovernanceError::NotOwner(other.to_string())
        );
    }

    // case 2. owner is not set
    {
        // first set owner to none
        update_owner(deps.as_mut().storage, None).unwrap();

        let res = execute_transfer_fee(deps.as_mut(), &other, transfer_fee);
        assert_eq!(res.unwrap_err(), GovernanceError::NoOwner);
    }
}

#[test]
fn executing_send_funds() {
    let mut deps = mock_dependencies();
    let [owner, other, origin, foo] = mock_addresses();
    let funds = coin(100, "uark");

    // case 1. owner is set
    {
        let info = mock_info(foo.as_str(), &vec![]);
        instantiate(
            deps.as_mut(),
            info,
            Some(owner.to_string()),
            Some(origin.to_string()),
            None,
        )
        .unwrap();

        // sender is owner
        let res = execute_send_funds(deps.as_mut(), &owner, foo.to_string(), funds.clone());
        assert!(res.is_ok());
        // sender is not owner
        let res = execute_send_funds(deps.as_mut(), &other, foo.to_string(), funds.clone());
        assert_eq!(
            res.unwrap_err(),
            GovernanceError::NotOwner(other.to_string())
        );
    }

    // case 2. owner is not set
    {
        // first set owner to none
        update_owner(deps.as_mut().storage, None).unwrap();

        let res = execute_send_funds(deps.as_mut(), &other, foo.to_string(), funds);
        assert_eq!(res.unwrap_err(), GovernanceError::NoOwner);
    }
}

#[test]
fn checking_paid() {
    let mut deps = mock_dependencies();
    let [owner, _, origin, foo] = mock_addresses();
    let funds = coin(100000, "uark");
    let info = mock_info(foo.as_str(), &vec![funds.clone()]);
    instantiate(
        deps.as_mut(),
        info,
        Some(owner.to_string()),
        Some(origin.to_string()),
        None,
    )
    .unwrap();

    // case 1. no transfer fee
    {
        // sender hasn't send funds
        let info = mock_info(foo.as_str(), &vec![]);
        let res = check_paid(&deps.storage, &info);
        assert!(res.is_ok());

        // sender can send transfer fee, even if none is defined
        let funds = coin(100000, "uark");
        let info = mock_info(foo.as_str(), &vec![funds.clone()]);
        let res = check_paid(&deps.storage, &info);
        assert!(res.is_ok());
    }

    // case 2. transfer fee is set
    {
        // set transfer fee
        let transfer_fee = coin(100, "uark");
        execute_transfer_fee(deps.as_mut(), &owner, Some(transfer_fee.clone())).unwrap();

        // sender hasn't send funds
        let info = mock_info(foo.as_str(), &vec![]);
        let res = check_paid(&deps.storage, &info);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            GovernanceError::IncorrectPaymentAmount(coin(0, "uark"), transfer_fee.clone())
        );

        // sender send transfer fee
        let funds = coin(100, "uark");
        let info = mock_info(foo.as_str(), &vec![funds.clone()]);
        let res = check_paid(&deps.storage, &info);
        assert!(res.is_ok());

        // sender send less than transfer fee
        let funds = coin(50, "uark");
        let info = mock_info(foo.as_str(), &vec![funds.clone()]);
        let res = check_paid(&deps.storage, &info);
        assert_eq!(
            res.unwrap_err(),
            GovernanceError::IncorrectPaymentAmount(funds, transfer_fee.clone())
        );

        // sender send more than transfer fee
        let funds = coin(150, "uark");
        let info = mock_info(foo.as_str(), &vec![funds.clone()]);
        let res = check_paid(&deps.storage, &info);
        assert_eq!(
            res.unwrap_err(),
            GovernanceError::IncorrectPaymentAmount(funds, transfer_fee.clone())
        );
    }
}
#[test]
fn into_attributes_works() {
    assert_eq!(
        Governance {
            owner: Some(Addr::unchecked("ark")),
            origin: Addr::unchecked("protocol"),
            transfer_fee: None,
        }
        .into_attributes(),
        vec![
            Attribute::new("owner", "ark"),
            Attribute::new("origin", "protocol"),
            Attribute::new("transfer_fee", "none")
        ],
    );
}

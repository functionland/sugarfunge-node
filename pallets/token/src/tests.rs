use crate::{mock::*, pallet::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_create_token_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Token::do_create_instance(&1, data));
        let uri = vec![0, 1];
        assert_ok!(Token::do_create_token(&1, 0, 2, true, uri));
        println!("token: {:?}", Tokens::<Test>::get(0, 2));
    })
}

#[test]
fn test_create_token_not_works() {
    new_test_ext().execute_with(|| {
        let uri = vec![0, 1];
        assert_noop!(
            Token::do_create_token(&1, 1, 2, true, uri),
            Error::<Test>::InvalidInstanceId
        );
        println!("token: {:?}", Tokens::<Test>::get(1, 2));
    })
}

#[test]
fn test_create_instance_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Token::do_create_instance(&1, data));
    })
}

#[test]
fn test_do_set_approval_for_all() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_set_approval_for_all(&1, &2, 1, true));
        assert_ok!(Token::do_set_approval_for_all(&1, &2, 1, false));
    })
}

#[test]
fn test_do_mint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_mint(&1, &2, 1, 2, 100));
    })
}

#[test]
fn test_do_batch_mint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Token::do_batch_mint(&1, &2, 1, token_ids, amounts));
    })
}

#[test]
fn test_do_burn_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_mint(&1, &2, 1, 2, 100));
        assert_ok!(Token::do_burn(&1, &2, 1, 2, 100));
    })
}

#[test]
fn test_do_batch_burn() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Token::do_batch_mint(
            &1,
            &2,
            1,
            token_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Token::do_batch_burn(&1, &2, 1, token_ids, amounts));
    })
}

#[test]
fn test_do_transfer_from() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_mint(&1, &1, 1, 2, 100));
        assert_ok!(Token::do_transfer_from(&1, &1, &2, 1, 2, 100));
    })
}

#[test]
fn test_do_batch_transfer_from() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Token::do_batch_mint(
            &1,
            &1,
            1,
            token_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Token::do_batch_transfer_from(
            &1, &1, &2, 1, token_ids, amounts
        ));
    })
}

#[test]
fn test_approved_or_owner() {
    new_test_ext().execute_with(|| {
        assert_eq!(Token::approved_or_owner(&1, &2, 1), false);
    })
}

#[test]
fn test_is_approved_for_all() {
    new_test_ext().execute_with(|| {
        assert_eq!(Token::is_approved_for_all(&1, &2, 1), false);
    })
}

#[test]
fn test_balance_of() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_mint(&1, &2, 1, 2, 100));
        assert_eq!(Token::balance_of(&2, 1, 2), 100);
    })
}

#[test]
fn test_balance_of_batch_token_ids_sample() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        let token_ids = vec![1; 3];
        let amounts = vec![100; 3];

        assert_ok!(Token::do_batch_mint(
            &1,
            &1,
            1,
            token_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Token::do_batch_mint(
            &1,
            &2,
            1,
            token_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Token::do_batch_mint(
            &1,
            &3,
            1,
            token_ids.clone(),
            amounts.clone()
        ));

        let account = vec![1, 2, 3];
        assert_eq!(
            Token::balance_of_batch(&account, 1, token_ids).unwrap(),
            vec![300; 3]
        );
    })
}

#[test]
fn test_balance_of_batch_token_ids_not_sample() {
    new_test_ext().execute_with(|| {
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        assert_ok!(Token::do_create_instance(&1, [0].to_vec()));
        let token_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];

        assert_ok!(Token::do_batch_mint(
            &1,
            &1,
            1,
            token_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Token::do_batch_mint(
            &1,
            &2,
            1,
            token_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Token::do_batch_mint(
            &1,
            &3,
            1,
            token_ids.clone(),
            amounts.clone()
        ));

        let account = vec![1, 2, 3];
        assert_eq!(
            Token::balance_of_batch(&account, 1, token_ids).unwrap(),
            amounts
        );
    })
}

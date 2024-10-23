#[cfg(test)]
mod tests {
    use crate::constants::DUMMY_DOMAIN;
    use crate::domain::EmailObject;
    use crate::routes::user::schemas::{
        AuthContextType, AuthenticationScope, BusinessAccount, CreateBusinessAccount,
        CreateUserAccount, CustomerType, DataSource, MerchantType, TradeType, UserType, VectorType,
    };
    use crate::routes::user::utils::{
        create_business_account, get_basic_business_account_by_user_id, get_stored_credentials,
        get_user, hard_delete_business_account, hard_delete_user_account, register_user,
        validate_business_account_active, verify_password,
    };
    use crate::schemas::{KycStatus, Status};
    use crate::utils::tests::get_test_pool;
    use secrecy::Secret;
    use sqlx::PgPool;
    use uuid::Uuid;
    #[tokio::test]
    async fn test_validate_active_business_account() {
        let mut business_account = BusinessAccount {
            id: Uuid::new_v4(),
            company_name: "SANU PRIVATE LIMITED".to_string(),
            vectors: vec![],
            kyc_status: KycStatus::Completed,
            is_active: Status::Active,
            is_deleted: false,
            verified: true,
            default_vector_type: VectorType::Gstin,
        };

        // Test case 2: KYC is pending
        business_account.kyc_status = KycStatus::Pending;
        business_account.is_active = Status::Active;
        business_account.is_deleted = false;
        business_account.verified = true;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(validate_response, Some("KYC is still pending".to_string()));

        // Test case 3: KYC is on-hold
        business_account.kyc_status = KycStatus::OnHold;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(validate_response, Some("KYC is On-hold".to_string()));

        // Test case 4: KYC is rejected
        business_account.kyc_status = KycStatus::Rejected;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(validate_response, Some("KYC is Rejected".to_string()));

        // Test case 5: Business account is inactive
        business_account.kyc_status = KycStatus::Completed;
        business_account.is_active = Status::Inactive;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(
            validate_response,
            Some("Business Account is inactive".to_string())
        );

        // Test case 6: Business account is deleted
        business_account.is_active = Status::Active;
        business_account.is_deleted = true;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(
            validate_response,
            Some("Business Account is deleted".to_string())
        );

        // Test case 7: Business user relation is not verified
        business_account.is_deleted = false;
        business_account.verified = false;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(
            validate_response,
            Some("Business User relation is not verified".to_string())
        );

        // Test case 8: All conditions are met
        business_account.verified = true;
        let validate_response = validate_business_account_active(&business_account);
        assert_eq!(validate_response, None);
    }

    #[tokio::test]
    async fn test_user_create_and_fetch() {
        let pool = get_test_pool().await;
        let mobile_no = "1234567890";
        let user_res = setup_user(
            &pool,
            "testuser1",
            "testuser@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        assert!(user_res.is_ok());
        assert!(get_user(vec![mobile_no], &pool).await.is_ok());
        let user_id = &user_res.unwrap();
        let user_res = setup_user(
            &pool,
            "testuser1",
            "testuser@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        assert!(user_res.is_err());
        let delete_res = hard_delete_user_account(&pool, &user_id.to_string()).await;
        assert!(delete_res.is_ok());
    }

    pub async fn setup_user(
        pool: &PgPool,
        username: &str,
        email: &str,
        mobile_no: &str,
        password: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let user_account = CreateUserAccount {
            username: username.to_string(),
            email: EmailObject::new(email.to_string()),
            mobile_no: mobile_no.to_string(),
            display_name: "Test User".to_string(),
            is_test_user: false,
            international_dialing_code: "+91".to_string(),
            source: DataSource::PlaceOrder,
            user_type: UserType::User,
            password: Secret::new(password.to_string()),
        };
        let user_result = register_user(pool, &user_account, DUMMY_DOMAIN).await?;
        Ok(user_result)
    }

    pub async fn setup_business(
        pool: &PgPool,
        mobile_no: &str,
        email: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let user_obj = get_user(vec![mobile_no], pool).await;
        let create_business_obj = CreateBusinessAccount {
            company_name: "Test Company".to_string(),
            is_test_account: false,
            customer_type: CustomerType::Buyer,
            source: DataSource::PlaceOrder,
            mobile_no: mobile_no.to_string(),
            email: EmailObject::new(email.to_string()),
            trade_type: vec![TradeType::Domestic],
            merchant_type: MerchantType::Manufacturer,
            opening_time: None,
            closing_time: None,
            proofs: vec![],
            default_vector_type: VectorType::Gstin,
        };
        let business_res_obj =
            create_business_account(pool, &user_obj.unwrap(), &create_business_obj, DUMMY_DOMAIN)
                .await?;
        Ok(business_res_obj)
    }

    #[tokio::test]
    async fn test_business_and_fetch() {
        let pool = get_test_pool().await;

        let mobile_no = "1234567892";
        let user_res = setup_user(
            &pool,
            "testuser3",
            "testuser3@example.com",
            mobile_no,
            "testuser@123",
        )
        .await;
        eprintln!("wwww{:?}", user_res);
        assert!(user_res.is_ok());
        let user_id = &user_res.unwrap();
        let business_res = setup_business(&pool, mobile_no, "business@example.com").await;
        assert!(business_res.is_ok());
        let fetch_business_obj_res = get_basic_business_account_by_user_id(user_id, &pool).await;
        assert!(fetch_business_obj_res.is_ok());
        let business_id = business_res.unwrap();
        let delete_bus_res = hard_delete_business_account(&pool, &business_id).await;
        assert!(delete_bus_res.is_ok());
        let delete_res = hard_delete_user_account(&pool, &mobile_no.to_string()).await;
        assert!(delete_res.is_ok());
    }

    #[tokio::test]
    async fn test_password_authentication() {
        let pool = get_test_pool().await;
        let passsword = "123";
        let mobile_no = "1234567893";
        let user_res = setup_user(
            &pool,
            "testuser4",
            "testuser4@example.com",
            mobile_no,
            passsword,
        )
        .await;
        assert!(user_res.is_ok());
        let auth_res = get_stored_credentials(
            &mobile_no,
            &AuthenticationScope::Password,
            &pool,
            AuthContextType::UserAccount,
        )
        .await;
        assert!(auth_res.is_ok());
        let auth_opt = auth_res.unwrap();
        assert!(auth_opt.is_some());
        let auth_obj = auth_opt.unwrap();
        let password_res = verify_password(Secret::new(passsword.to_string()), &auth_obj).await;
        assert!(password_res.is_ok());
        let password_res = verify_password(Secret::new("abc".to_string()), &auth_obj).await;
        assert!(password_res.is_err());
        let delete_res = hard_delete_user_account(&pool, &mobile_no.to_string()).await;
        assert!(delete_res.is_ok());
    }
}

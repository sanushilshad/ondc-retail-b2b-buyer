#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use crate::configuration::get_configuration;
    use crate::constants::DUMMY_DOMAIN;
    use crate::routes::order::schemas::{PaymentSettlementPhase, PaymentSettlementType};
    use crate::schemas::{FeeType, KycStatus, RegisteredNetworkParticipant, Status};
    use crate::startup::get_connection_pool;
    use crate::user_client::{BusinessAccount, MaskingType, UserAccount, UserVector, VectorType};
    use crate::utils::validate_business_account_active;
    use bigdecimal::BigDecimal;
    use sqlx::PgPool;
    use uuid::Uuid;
    pub async fn get_test_pool() -> PgPool {
        let mut configuration = get_configuration().expect("Failed to read configuration.");
        // configuration.database.name = TEST_DB.to_string();
        configuration.application.port = 0;
        get_connection_pool(&configuration.database)
    }

    pub fn get_dummy_user_account(
        username: String,
        mobile_no: String,
        email: String,
    ) -> UserAccount {
        UserAccount {
            id: Uuid::new_v4(),
            username: username,
            mobile_no: mobile_no,
            email: email,
            is_active: Status::Active,
            display_name: "SANU SHILSHAD".to_owned(),
            vectors: vec![],
            international_dialing_code: "+91".to_owned(),
            user_account_number: "123445".to_owned(),
            alt_user_account_number: "123445".to_owned(),
            is_test_user: true,
            is_deleted: false,
            user_role: "user".to_owned(),
        }
    }

    pub fn get_dummy_business_account() -> BusinessAccount {
        let vector = UserVector {
            key: VectorType::PanCardNo,
            value: "CKWPC9262N".to_owned(),
            masking: MaskingType::PartialMask,
            verified: true,
        };
        BusinessAccount {
            id: Uuid::new_v4(),
            company_name: "SANU SHILSHAD".to_owned(),
            vectors: vec![vector],
            kyc_status: KycStatus::Completed,
            is_active: Status::Active,
            is_deleted: false,
            verified: true,
            default_vector_type: VectorType::PanCardNo,
            proofs: vec![],
        }
    }

    pub fn get_dummy_registed_np_detail() -> RegisteredNetworkParticipant {
        RegisteredNetworkParticipant {
            code: "SANU".to_owned(),
            name: "SANU".to_owned(),
            logo: "google.com".to_owned(),
            signing_key: "google.com".to_owned().into(),
            id: Uuid::new_v4(),
            subscriber_id: DUMMY_DOMAIN.to_string(),
            subscriber_uri: format!("{}/v1/ondc/seller", DUMMY_DOMAIN),
            long_description: "SANU".to_owned(),
            short_description: "SANU".to_owned(),
            fee_type: FeeType::Amount,
            fee_value: BigDecimal::from_str("0.0").unwrap(),
            unique_key_id: "SANU".to_owned(),
            settlement_phase: PaymentSettlementPhase::SaleAmount,
            settlement_type: PaymentSettlementType::Neft,
            bank_account_no: "1234567890".to_owned(),
            bank_ifsc_code: "HDFC0000102".to_owned(),
            bank_beneficiary_name: "SANU SHILSHAD".to_owned(),
            bank_name: "SANU BANK".to_owned(),
        }
    }

    #[actix_web::test]
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
            proofs: vec![],
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
}

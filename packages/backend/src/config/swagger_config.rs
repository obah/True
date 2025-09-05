use crate::authenticity::get_certificate::CertificateResponse;
use crate::authenticity::get_manufacturer::__path_get_manufacturer;
use crate::authenticity::is_username_exist::__path_manufacturer_name_exists;
use crate::authenticity::is_username_exist::{IsExistsQuery, IsExistsResponse};
use crate::certificate::{__path_get_certificate,__path_save_certificate, Certificates, CertificateDTO};
use crate::contract_models::{Manufacturer, ManufacturerQuery, Item};
use crate::models::certificate_model::{
    CertificateData, Eip712Object, RegInput, SignedCertificate,
};
use crate::ownership::{
    get_user_info::{__path_get_user, UserQuery, UserResponse},
    is_name_exist::{__path_user_exists, UserExistsQuery, UserExistsResponse},
    get_my_items::{__path_get_owner_items, ItemQuery, ItemsResponse},
    transfer_ownership_code::{__path_transfer_ownership_code, OwnershipCodeResponse, GenerateOwnershipCodeQuery},
    get_transfer_code::{__path_get_ownership_code, GetOwnershipCodeQuery},
    revoke_ownership_code::{__path_revoke_ownership_code, OwnershipQuery, OwnershipResponse },
    get_item::__path_get_item,
    check_before_claim::{__path_check_before_claim, OwnershipCheckQuery, OwnershipCheckResponse}
};
use crate::services::{
    create_eip712::__path_create_certificate,
    other_tests::{
        __path_generate_signature, __path_get_owner, __path_manufacturer_registers,
        __path_verify_signature,
    },
    qr_code::__path_generate_qr_code,
    verify_authenticity::__path_verify_authenticity,
    set_autheticity::{__path_set_authenticity, SetAuthenticityResponse, SetAuthenticityRequest},
    claim_ownership::{__path_claim_ownership, ClaimOwnershipResponse, ClaimOwnershipRequest},
    create_item::{__path_create_item, CreateItemResponse, CreateItemRequest},
};
use crate::sync::{__path_sync, SyncPayload, SyncResponse};
use crate::ownership::batch_items::{__path_batch_items, BatchItemsPayload, BatchItemsResponse};
use crate::services::register_user::{__path_user_register, UserRegisterResponse, UserRegisterRequest};
use utoipa::OpenApi;

// Swagger/OpenAPI configuration
#[derive(OpenApi)]
#[openapi(
    paths(
        verify_authenticity,
        generate_signature,
        manufacturer_registers,
        get_owner,
        verify_signature,
        create_certificate,
        generate_qr_code,
        get_manufacturer,
        manufacturer_name_exists,
        get_user,
        user_exists,
        get_owner_items,
        transfer_ownership_code,
        get_ownership_code,
        revoke_ownership_code,
        user_register,
        set_authenticity,
        claim_ownership,
        create_item,
        get_item,
        sync,
        batch_items,
        get_certificate,
        save_certificate,
        check_before_claim
    ),
    components(
        schemas(
            RegInput,
            CertificateData,
            SignedCertificate,
            Eip712Object,
            ManufacturerQuery,
            Manufacturer,
            IsExistsResponse,
            IsExistsQuery,
            UserResponse,
            UserQuery,
            UserExistsQuery,
            UserExistsResponse,
            ItemQuery,
            ItemsResponse,
            GenerateOwnershipCodeQuery,
            OwnershipCodeResponse,
            GetOwnershipCodeQuery,
            OwnershipResponse,
            OwnershipQuery,
            UserRegisterResponse,
            UserRegisterRequest,
            SetAuthenticityRequest,
            SetAuthenticityResponse,
            ClaimOwnershipRequest,
            ClaimOwnershipResponse,
            CreateItemResponse,
            CreateItemRequest,
            Item,
            SyncPayload, SyncResponse, BatchItemsResponse, BatchItemsPayload,
            CertificateDTO, Certificates,
            OwnershipCheckResponse, OwnershipCheckQuery
        ),
        // responses()
    ),
    tags(
        (name = "ERI", description = "Signature Verifying APIs")
    ),
    info(
        title = "ERI APIs",
        description = "Signature Verifying Project on the Blockchain",
    ),


    // security(
    //     (),
    //     ("my_auth" = ["read:items", "edit:items"]),
    //     ("token_jwt" = [])
    // ),
    // servers(
    //     (url = "http://localhost:8989", description = "Local server"),
    //     (url = "http://api.{username}:{port}", description = "Remote API",
    //         variables(
    //             ("username" = (default = "demo", description = "Default username for API")),
    //             ("port" = (default = "8080", enum_values("8080", "5000", "3030"), description = "Supported ports for API"))
    //         )
    //     )
    // )
)]
pub struct ApiDoc;

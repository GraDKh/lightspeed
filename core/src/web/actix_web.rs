use crate::error::{LightSpeedError, WebErrorDetails};
use crate::service::auth::{Auth, AuthContext, AuthService, RolesProvider};
use crate::service::jwt::JwtService;
use actix_web_external::dev::HttpResponseBuilder;
use actix_web_external::{http, HttpRequest, HttpResponse, ResponseError};
use log::*;

pub const JWT_TOKEN_HEADER: &str = "Authorization";
pub const JWT_TOKEN_HEADER_SUFFIX: &str = "Bearer ";

// This must be equal to JWT_TOKEN_HEADER_SUFFIX.len()
pub const JWT_TOKEN_HEADER_SUFFIX_LEN: usize = 7;

#[derive(Clone)]
pub struct WebAuthService<T: RolesProvider> {
    auth_service: AuthService<T>,
    jwt_service: JwtService,
}

impl<T: RolesProvider> WebAuthService<T> {
    pub fn new(auth_service: AuthService<T>, jwt_service: JwtService) -> Self {
        Self {
            auth_service,
            jwt_service,
        }
    }

    pub fn token_string_from_request<'a>(
        &self,
        req: &'a HttpRequest,
    ) -> Result<&'a str, LightSpeedError> {
        if let Some(header) = req.headers().get(JWT_TOKEN_HEADER) {
            return header
                .to_str()
                .map_err(|err| LightSpeedError::ParseAuthHeaderError {
                    message: format!("{}", err),
                })
                .and_then(|header| {
                    trace!("Token found in request: [{}]", header);
                    if header.len() > JWT_TOKEN_HEADER_SUFFIX_LEN {
                        Ok(&header[JWT_TOKEN_HEADER_SUFFIX_LEN..])
                    } else {
                        Err(LightSpeedError::ParseAuthHeaderError {
                            message: format!("Unexpected auth header: {}", header),
                        })
                    }
                });
        };
        Err(LightSpeedError::MissingAuthTokenError)
    }

    pub fn token_from_auth(&self, auth: &Auth) -> Result<String, LightSpeedError> {
        Ok(self.jwt_service.generate_from_payload(auth)?.1)
    }

    pub fn auth_from_request(&self, req: &HttpRequest) -> Result<AuthContext, LightSpeedError> {
        self.token_string_from_request(req)
            .and_then(|token| self.auth_from_token_string(token))
    }

    pub fn auth_from_token_string(&self, token: &str) -> Result<AuthContext, LightSpeedError> {
        let auth = self.jwt_service.parse_payload::<Auth>(token);
        trace!("Auth built from request: [{:?}]", auth);
        Ok(self.auth_service.auth(auth?))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::config::JwtConfig;
    use crate::service::auth::{InMemoryRolesProvider, Role};
    use crate::service::jwt::JWT;
    use actix_service::Service;
    use actix_web_external::test::{init_service, TestRequest};
    use actix_web_external::{http::StatusCode, web, App};
    use jsonwebtoken::Algorithm;

    #[actix_rt::test]
    async fn access_protected_url_should_return_unauthorized_if_no_token() {
        // Arrange
        let mut srv = init_service(App::new().service(web::resource("/auth").to(username))).await;

        let request = TestRequest::get().uri("/auth").to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn access_protected_url_should_return_unauthorized_if_expired_token() {
        // Arrange
        let token = JWT {
            payload: Auth {
                username: "Amelia".to_owned(),
                id: 100,
                roles: vec![],
            },
            exp: 0,
            iat: 0,
            sub: "".to_owned(),
        };
        let token = new_service()
            .jwt_service
            .generate_from_token(&token)
            .unwrap();

        let mut srv = init_service(App::new().service(web::resource("/auth").to(username))).await;

        let request = TestRequest::get()
            .uri("/auth")
            .header(
                JWT_TOKEN_HEADER,
                format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token),
            )
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn access_protected_url_should_return_ok_if_valid_token() {
        // Arrange
        let auth = Auth {
            username: "Amelia".to_owned(),
            id: 100,
            roles: vec![],
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let mut srv = init_service(App::new().service(web::resource("/auth").to(username))).await;

        let request = TestRequest::get()
            .uri("/auth")
            .header(
                JWT_TOKEN_HEADER,
                format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token),
            )
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn access_admin_url_should_return_forbidden_if_not_admin_role() {
        // Arrange
        let auth = Auth {
            username: "Amelia".to_owned(),
            id: 100,
            roles: vec![],
        };
        let token = new_service().token_from_auth(&auth).unwrap();

        let mut srv = init_service(App::new().service(web::resource("/auth").to(admin))).await;

        let request = TestRequest::get()
            .uri("/auth")
            .header(
                JWT_TOKEN_HEADER,
                format!("{}{}", JWT_TOKEN_HEADER_SUFFIX, token),
            )
            .to_request();

        // Act
        let resp = srv.call(request).await.unwrap();

        // Assert
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    async fn admin(req: HttpRequest) -> actix_web_external::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        auth_context.has_role("admin")?;
        Ok(auth_context.auth.username.clone())
    }

    async fn username(req: HttpRequest) -> actix_web_external::Result<String> {
        let auth_service = new_service();
        let auth_context = auth_service.auth_from_request(&req)?;
        Ok(auth_context.auth.username)
    }

    fn new_service() -> WebAuthService<InMemoryRolesProvider> {
        WebAuthService {
            auth_service: AuthService::new(InMemoryRolesProvider::new(vec![Role {
                name: "admin".to_owned(),
                permissions: vec![],
            }])),
            jwt_service: JwtService::new(&JwtConfig {
                secret: "secret".to_owned(),
                signature_algorithm: Algorithm::HS256,
                token_validity_minutes: 10,
            })
            .unwrap(),
        }
    }
}

impl ResponseError for LightSpeedError {
    fn error_response(&self) -> HttpResponse {
        match self {
            LightSpeedError::InvalidTokenError { .. }
            | LightSpeedError::ExpiredTokenError { .. }
            | LightSpeedError::GenerateTokenError { .. }
            | LightSpeedError::MissingAuthTokenError { .. }
            | LightSpeedError::ParseAuthHeaderError { .. }
            | LightSpeedError::UnauthenticatedError => {
                warn!("User is unauthorized: {}", self);
                HttpResponse::Unauthorized().finish()
            }
            LightSpeedError::ForbiddenError { .. } => {
                warn!("Access is forbidden: {}", self);
                HttpResponse::Forbidden().finish()
            }
            LightSpeedError::InternalServerError { message } => {
                error!("Internal server error: {}", message);
                HttpResponse::InternalServerError().finish()
            }
            LightSpeedError::ValidationError { details } => {
                warn!("ValidationError: {}", self);
                let http_code = http::StatusCode::UNPROCESSABLE_ENTITY;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_error_details(
                    http_code.as_u16(),
                    details,
                ))
            }
            LightSpeedError::BadRequest { .. } => {
                warn!("BadRequest: {}", self);
                HttpResponse::BadRequest().finish()
            }
            LightSpeedError::RequestConflict { code, .. } => {
                error!("RequestConflict: {}", self);
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(
                    http_code.as_u16(),
                    &Some((*code).to_string()),
                ))
            }
            LightSpeedError::ServiceUnavailable { code, .. } => {
                error!("ServiceUnavailable: {}", self);
                let http_code = http::StatusCode::CONFLICT;
                HttpResponseBuilder::new(http_code).json(WebErrorDetails::from_message(
                    http_code.as_u16(),
                    &Some((*code).to_string()),
                ))
            }
            LightSpeedError::ModuleBuilderError { .. }
            | LightSpeedError::ModuleStartError { .. }
            | LightSpeedError::ConfigurationError { .. }
            | LightSpeedError::PasswordEncryptionError { .. }
            | LightSpeedError::RepositoryError { .. } => {
                error!("InternalServerError: {}", self);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
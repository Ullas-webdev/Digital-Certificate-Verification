#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, String, Address, symbol_short, Symbol};

// Certificate structure containing all essential information
#[contracttype]
#[derive(Clone)]
pub struct Certificate {
    pub cert_id: u64,
    pub student_name: String,
    pub course_name: String,
    pub institution: String,
    pub issue_date: u64,
    pub issuer: Address,
    pub is_valid: bool,
}

// Mapping certificate ID to Certificate data
#[contracttype]
pub enum CertificateBook {
    Cert(u64)
}

// Counter for generating unique certificate IDs
const CERT_COUNT: Symbol = symbol_short!("CERT_CNT");

#[contract]
pub struct CertificateContract;

#[contractimpl]
impl CertificateContract {
    
    /// Issue a new certificate
    /// Only authorized issuers (institutions) can call this function
    pub fn issue_certificate(
        env: Env,
        issuer: Address,
        student_name: String,
        course_name: String,
        institution: String
    ) -> u64 {
        // Require authentication from issuer
        issuer.require_auth();
        
        // Get and increment certificate counter
        let mut cert_count: u64 = env.storage().instance().get(&CERT_COUNT).unwrap_or(0);
        cert_count += 1;
        
        // Get current timestamp
        let issue_date = env.ledger().timestamp();
        
        // Create new certificate
        let certificate = Certificate {
            cert_id: cert_count,
            student_name,
            course_name,
            institution,
            issue_date,
            issuer: issuer.clone(),
            is_valid: true,
        };
        
        // Store certificate
        env.storage().instance().set(&CertificateBook::Cert(cert_count), &certificate);
        env.storage().instance().set(&CERT_COUNT, &cert_count);
        
        // Extend TTL for data persistence
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Certificate issued with ID: {}", cert_count);
        
        cert_count
    }
    
    /// Verify a certificate by its ID
    /// Anyone can call this function to verify authenticity
    pub fn verify_certificate(env: Env, cert_id: u64) -> Certificate {
        let key = CertificateBook::Cert(cert_id);
        
        env.storage().instance().get(&key).unwrap_or(Certificate {
            cert_id: 0,
            student_name: String::from_str(&env, "Not_Found"),
            course_name: String::from_str(&env, "Not_Found"),
            institution: String::from_str(&env, "Not_Found"),
            issue_date: 0,
            issuer: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            is_valid: false,
        })
    }
    
    /// Revoke a certificate
    /// Only the original issuer can revoke their certificate
    pub fn revoke_certificate(env: Env, issuer: Address, cert_id: u64) {
        // Require authentication from issuer
        issuer.require_auth();
        
        let key = CertificateBook::Cert(cert_id);
        let mut certificate: Certificate = env.storage().instance().get(&key)
            .expect("Certificate not found");
        
        // Verify that the caller is the original issuer
        if certificate.issuer != issuer {
            panic!("Only the original issuer can revoke this certificate");
        }
        
        // Mark certificate as invalid
        certificate.is_valid = false;
        
        // Update storage
        env.storage().instance().set(&key, &certificate);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Certificate {} has been revoked", cert_id);
    }
    
    /// Get total number of certificates issued
    pub fn get_total_certificates(env: Env) -> u64 {
        env.storage().instance().get(&CERT_COUNT).unwrap_or(0)
    }
}
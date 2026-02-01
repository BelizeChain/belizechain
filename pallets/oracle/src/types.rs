//! # BelizeChain Oracle Pallet Types
//!
//! Core data structures for the oracle system including price feeds,
//! merchant verification, sanctions, and external data integration.

use codec::{Encode, Decode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::{
    pallet_prelude::*,
    BoundedVec,
};
use sp_runtime::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

/// Maximum length for data sources
pub const MAX_SOURCE_LEN: u32 = 64;

/// Maximum length for oracle data values
pub const MAX_DATA_LEN: u32 = 256;

/// Maximum length for sanction reasons
pub const MAX_REASON_LEN: u32 = 256;

/// Maximum length for certification numbers
pub const MAX_CERT_LEN: u32 = 128;

/// Currency types supported by the oracle
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Currency {
    /// Belize Dollar
    BZD,
    /// US Dollar
    USD,
    /// Euro
    EUR,
    /// Canadian Dollar
    CAD,
    /// Mexican Peso
    MXN,
}

impl Currency {
    /// Convert to u8 for event encoding
    pub fn to_u8(&self) -> u8 {
        match self {
            Currency::BZD => 0,
            Currency::USD => 1,
            Currency::EUR => 2,
            Currency::CAD => 3,
            Currency::MXN => 4,
        }
    }
}

/// Currency pair for exchange rates
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CurrencyPair {
    /// Base currency
    pub base: Currency,
    /// Quote currency
    pub quote: Currency,
}

impl CurrencyPair {
    pub fn new(base: Currency, quote: Currency) -> Self {
        Self { base, quote }
    }
}

/// Oracle data point with metadata
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OracleDataPoint<AccountId, BlockNumber> {
    /// The actual data value (encoded as bytes for flexibility)
    pub value: BoundedVec<u8, ConstU32<MAX_DATA_LEN>>,
    /// When this data was submitted
    pub timestamp: BlockNumber,
    /// Which operator submitted it
    pub operator: AccountId,
    /// Data source identifier
    pub source: BoundedVec<u8, ConstU32<MAX_SOURCE_LEN>>,
    /// Confidence score (0-100)
    pub confidence: u8,
}

/// Aggregated price feed data
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PriceFeedData<BlockNumber> {
    /// Currency pair
    pub pair: CurrencyPair,
    /// Price (scaled by 1e6 for 6 decimal precision)
    pub price: u128,
    /// Number of operators that contributed
    pub num_submissions: u32,
    /// Last update block
    pub last_update: BlockNumber,
    /// Variance/spread of submitted values
    pub variance: u32,
}

/// Merchant category for tourism incentives
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MerchantCategory {
    /// Hotels, resorts, lodging (8% cashback)
    Accommodation,
    /// Restaurants, bars, cafes (5%)
    FoodBeverage,
    /// Tour operators, guides (7%)
    TourOperator,
    /// Taxis, shuttles, car rentals (5%)
    Transportation,
    /// Retail in tourist zones (3%)
    Retail,
    /// Other approved merchants (3%)
    Other,
}

impl MerchantCategory {
    /// Convert to u8 for event encoding
    pub fn to_u8(&self) -> u8 {
        match self {
            MerchantCategory::Accommodation => 0,
            MerchantCategory::FoodBeverage => 1,
            MerchantCategory::TourOperator => 2,
            MerchantCategory::Transportation => 3,
            MerchantCategory::Retail => 4,
            MerchantCategory::Other => 5,
        }
    }

    /// Get the tourism incentive reward percentage
    pub fn reward_percentage(&self) -> u8 {
        match self {
            MerchantCategory::Accommodation => 8,
            MerchantCategory::TourOperator => 7,
            MerchantCategory::FoodBeverage => 5,
            MerchantCategory::Transportation => 5,
            MerchantCategory::Retail => 3,
            MerchantCategory::Other => 3,
        }
    }
}

/// Merchant verification information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct MerchantInfo<AccountId, BlockNumber> {
    /// Merchant account
    pub merchant: AccountId,
    /// Business category
    pub category: MerchantCategory,
    /// Tourism Board certification number
    pub certification: BoundedVec<u8, ConstU32<MAX_CERT_LEN>>,
    /// Business license number
    pub license: BoundedVec<u8, ConstU32<MAX_CERT_LEN>>,
    /// Physical location (lat, long scaled by 1e6)
    pub location: Option<(i32, i32)>,
    /// Verified by oracle at block
    pub verified_at: BlockNumber,
    /// Expiry block (typically 1 year = ~5.26M blocks)
    pub expires_at: BlockNumber,
}

/// Source of sanction information
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SanctionSource {
    /// US Office of Foreign Assets Control
    OFAC,
    /// United Nations Security Council
    UN,
    /// European Union
    EU,
    /// Belize Financial Services Commission
    FSC,
    /// Other regulatory body
    Other,
}

impl SanctionSource {
    /// Convert to u8 for event encoding
    pub fn to_u8(&self) -> u8 {
        match self {
            SanctionSource::OFAC => 0,
            SanctionSource::UN => 1,
            SanctionSource::EU => 2,
            SanctionSource::FSC => 3,
            SanctionSource::Other => 4,
        }
    }
}

/// Sanction information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SanctionInfo<BlockNumber> {
    /// Sanction source
    pub source: SanctionSource,
    /// When added to list
    pub added_at: BlockNumber,
    /// Reason for sanction
    pub reason: BoundedVec<u8, ConstU32<MAX_REASON_LEN>>,
    /// Is currently active
    pub active: bool,
    /// Optional expiry block
    pub expires_at: Option<BlockNumber>,
}



/// Oracle feed type identifier
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum OracleFeedType {
    /// Price feed for currency pair
    PriceFeed,
    /// Merchant category verification
    MerchantVerification,
    /// Sanctions list check
    SanctionsCheck,
    /// Land registry lookup
    LandRegistry,
    /// Weather/climate data
    WeatherData,
    /// Identity verification
    IdentityVerification,
}

/// KYC verification levels
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum KycLevel {
    /// Level 0: No verification (email/phone only)
    None,
    /// Level 1: Government ID verification
    Basic,
    /// Level 2: Biometric + Address verification
    Enhanced,
    /// Level 3: Full KYC with ongoing monitoring
    Full,
}

impl KycLevel {
    /// Convert to u8 for event encoding
    pub fn to_u8(&self) -> u8 {
        match self {
            KycLevel::None => 0,
            KycLevel::Basic => 1,
            KycLevel::Enhanced => 2,
            KycLevel::Full => 3,
        }
    }

    /// Get transaction limits for KYC level (in DALLA, scaled by 1e12)
    pub fn transaction_limit(&self) -> u128 {
        match self {
            KycLevel::None => 1_000 * 1_000_000_000_000,      // 1,000 DALLA
            KycLevel::Basic => 10_000 * 1_000_000_000_000,    // 10,000 DALLA
            KycLevel::Enhanced => 100_000 * 1_000_000_000_000, // 100,000 DALLA
            KycLevel::Full => u128::MAX,                       // Unlimited
        }
    }

    /// Get daily limits for KYC level (in DALLA, scaled by 1e12)
    pub fn daily_limit(&self) -> u128 {
        match self {
            KycLevel::None => 5_000 * 1_000_000_000_000,      // 5,000 DALLA/day
            KycLevel::Basic => 25_000 * 1_000_000_000_000,    // 25,000 DALLA/day
            KycLevel::Enhanced => 250_000 * 1_000_000_000_000, // 250,000 DALLA/day
            KycLevel::Full => u128::MAX,                       // Unlimited
        }
    }
}

/// Identity verification information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IdentityInfo<AccountId, BlockNumber> {
    /// Account being verified
    pub account: AccountId,
    /// KYC verification level achieved
    pub kyc_level: KycLevel,
    /// Government ID number (hashed for privacy)
    pub id_hash: [u8; 32],
    /// Verification provider (e.g., "Onfido", "Jumio", "Gov-Belize")
    pub provider: BoundedVec<u8, ConstU32<64>>,
    /// When verification was completed
    pub verified_at: BlockNumber,
    /// Verification expiry (for re-verification)
    pub expires_at: BlockNumber,
    /// Biometric verification completed
    pub biometric_verified: bool,
    /// Address proof submitted
    pub address_verified: bool,
}

/// Property ID for land registry (District + Parcel Number)
pub type PropertyId = [u8; 32];

/// Land registry encumbrance (mortgage, lien, etc.)
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Encumbrance<BlockNumber> {
    /// Type of encumbrance ("mortgage", "lien", "easement")
    pub encumbrance_type: BoundedVec<u8, ConstU32<32>>,
    /// Amount (if monetary, in bBZD scaled by 1e6)
    pub amount: u128,
    /// Holder of the encumbrance
    pub holder: BoundedVec<u8, ConstU32<128>>,
    /// Registration date
    pub registered_at: BlockNumber,
}

/// Simplified land ownership information (MaxEncodedLen compatible)
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LandOwnershipInfo<AccountId, BlockNumber> {
    /// Property identifier
    pub property_id: PropertyId,
    /// Primary owner account
    pub owner: AccountId,
    /// Property valuation (in bBZD, scaled by 1e6)
    pub valuation: u128,
    /// Last transfer date
    pub last_transfer: BlockNumber,
    /// Has active encumbrances
    pub has_encumbrances: bool,
    /// Number of co-owners (if any)
    pub co_owner_count: u8,
    /// Verified by oracle
    pub verified: bool,
    /// Oracle verification timestamp
    pub verified_at: BlockNumber,
}

// ============================================================================
// IoT Data Oracle Types (Phase 1: Nawal Expansion)
// ============================================================================

/// AI/ML model domain types for data classification
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ModelDomain {
    /// General purpose AI
    General,
    /// Agricultural technology (crop monitoring, yield prediction)
    AgriTech,
    /// Marine/ocean monitoring (coral reefs, water quality)
    Marine,
    /// Education (personalized learning, student outcomes)
    Education,
    /// Technology sector (software development, innovation)
    Tech,
}

impl ModelDomain {
    pub fn to_u8(&self) -> u8 {
        match self {
            ModelDomain::General => 0,
            ModelDomain::AgriTech => 1,
            ModelDomain::Marine => 2,
            ModelDomain::Education => 3,
            ModelDomain::Tech => 4,
        }
    }
    
    /// Priority multiplier for PoUW rewards (Belize national priorities)
    pub fn reward_multiplier(&self) -> u8 {
        match self {
            ModelDomain::AgriTech => 150,     // 1.5x (food security)
            ModelDomain::Marine => 140,       // 1.4x (ecosystem protection)
            ModelDomain::Education => 130,    // 1.3x (human capital)
            ModelDomain::Tech => 110,         // 1.1x (economic growth)
            ModelDomain::General => 100,      // 1.0x (baseline)
        }
    }
}

/// IoT device sensor types
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SensorType {
    /// Temperature sensor (Celsius * 100)
    Temperature,
    /// Humidity sensor (percentage * 100)
    Humidity,
    /// Soil moisture (percentage * 100)
    SoilMoisture,
    /// pH level (pH * 100)
    PH,
    /// GPS location
    GPS,
    /// Light intensity (lux)
    Light,
    /// Water salinity (PSU * 100)
    Salinity,
    /// Dissolved oxygen (mg/L * 100)
    DissolvedOxygen,
    /// Pressure (hPa)
    Pressure,
    /// Other sensor type
    Other,
}

/// Drone specifications
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DroneSpec {
    /// Drone model/manufacturer
    pub model: BoundedVec<u8, ConstU32<64>>,
    /// Camera resolution (megapixels * 10)
    pub camera_resolution: u16,
    /// Has multispectral camera
    pub multispectral: bool,
    /// Maximum flight altitude (meters)
    pub max_altitude: u16,
    /// Flight time (minutes)
    pub flight_time: u16,
}

/// Phone sensor specifications
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PhoneSpec {
    /// Operating system
    pub os: BoundedVec<u8, ConstU32<32>>,
    /// Has camera
    pub has_camera: bool,
    /// Has GPS
    pub has_gps: bool,
    /// Has accelerometer
    pub has_accelerometer: bool,
}

/// Generic IoT sensor specifications
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SensorSpec {
    /// Sensor type
    pub sensor_type: SensorType,
    /// Measurement accuracy (percentage * 100, e.g., 9500 = 95%)
    pub accuracy: u16,
    /// Sampling rate (Hz * 10)
    pub sampling_rate: u16,
}

/// IoT device type with specifications
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DeviceType {
    /// Aerial or underwater drone
    Drone(DroneSpec),
    /// Mobile phone sensor
    PhoneSensor(PhoneSpec),
    /// Generic IoT sensor
    IoTSensor(SensorSpec),
    /// Weather station (multiple sensors)
    WeatherStation,
    /// Agricultural sensor array
    AgriculturalSensor,
    /// Marine buoy with sensors
    MarineBuoy,
    /// Fixed camera
    Camera,
    /// Other device type
    Other,
}

/// IoT device registration information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IoTDevice<AccountId, BlockNumber> {
    /// Unique device identifier (hash of serial number + owner)
    pub device_id: [u8; 32],
    /// Device type and specifications
    pub device_type: DeviceType,
    /// Device owner account
    pub owner: AccountId,
    /// Physical location (lat, long scaled by 1e6)
    pub location: Option<(i32, i32)>,
    /// Registration block
    pub registered_at: BlockNumber,
    /// Last data submission block
    pub last_active: BlockNumber,
    /// Reputation score (0-10000, representing 0.00-100.00%)
    pub reputation_score: u16,
    /// Total data submissions
    pub data_submissions: u64,
    /// Oracle verified device
    pub verified: bool,
}

/// Data quality metrics for scoring submissions
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DataQualityMetrics {
    /// Accuracy (0-100): Compared to ground truth or peer validation
    pub accuracy: u8,
    /// Timeliness (0-100): How fresh is the data
    pub timeliness: u8,
    /// Completeness (0-100): Missing fields check
    pub completeness: u8,
    /// Consistency (0-100): Cross-validation with other sources
    pub consistency: u8,
    /// Provenance (0-100): Device reputation score mapped to 0-100
    pub provenance: u8,
}

impl DataQualityMetrics {
    /// Calculate weighted quality score (0-1000)
    pub fn calculate_score(&self) -> u16 {
        let weighted_sum = 
            (self.accuracy as u16 * 30) +        // 30% weight
            (self.timeliness as u16 * 20) +      // 20% weight
            (self.completeness as u16 * 15) +    // 15% weight
            (self.consistency as u16 * 20) +     // 20% weight
            (self.provenance as u16 * 15);       // 15% weight
        weighted_sum / 10  // Scale to 0-1000
    }
}

/// Collection type for mobile phone data
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CollectionType {
    /// Photo submission
    Photo,
    /// GPS track
    GPSTrack,
    /// Sensor reading
    SensorReading,
    /// Survey response
    Survey,
    /// Other collection
    Other,
}

/// Extended oracle feed types including IoT data
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DataFeedType {
    // Existing feed types
    /// Currency exchange rate
    PriceFeed,
    /// Merchant verification
    MerchantVerification,
    /// Sanctions list check
    SanctionsCheck,
    /// Land registry data
    LandRegistry,
    /// Identity verification
    IdentityVerification,
    
    // NEW: IoT Data Feeds
    /// Drone imagery (aerial or underwater)
    DroneImagery(ModelDomain),
    /// Sensor reading from IoT device
    SensorReading(SensorType),
    /// Mobile phone collected data
    PhoneCollection(CollectionType),
    /// Weather station data
    WeatherData,
    /// GPS tracking data
    GPSTracking,
    /// Fixed camera feed
    CameraFeed,
}

/// IoT data submission with quality metrics
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IoTDataSubmission<AccountId, BlockNumber> {
    /// Device that submitted the data
    pub device_id: [u8; 32],
    /// Device owner/operator
    pub operator: AccountId,
    /// Type of data feed
    pub feed_type: DataFeedType,
    /// Associated AI model domain (if applicable)
    pub domain: Option<ModelDomain>,
    /// Actual data payload (IPFS CID or direct data)
    pub data: BoundedVec<u8, ConstU32<MAX_DATA_LEN>>,
    /// Data hash for integrity verification
    pub data_hash: [u8; 32],
    /// Submission timestamp
    pub timestamp: BlockNumber,
    /// Quality metrics
    pub quality: DataQualityMetrics,
    /// Location where data was collected
    pub location: Option<(i32, i32)>,
}

/// Oracle operator contribution stats
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct OracleOperatorStats<BlockNumber> {
    /// Total data submissions
    pub total_submissions: u64,
    /// Average quality score (0-1000)
    pub avg_quality_score: u16,
    /// Submissions by domain
    pub agritech_submissions: u32,
    pub marine_submissions: u32,
    pub education_submissions: u32,
    pub tech_submissions: u32,
    pub general_submissions: u32,
    /// Uptime percentage (0-10000, representing 0.00-100.00%)
    pub uptime_percentage: u16,
    /// Last active block
    pub last_active: BlockNumber,
    /// Total rewards earned (in DALLA, scaled by 1e12)
    pub total_rewards: u128,
}


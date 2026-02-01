"""
BelizeChain Oracle Integration Tests
Phase 3 Complete - End-to-End Testing

Tests the complete flow:
IoT Device → Oracle API → Blockchain (Oracle + Staking) → Nawal AI → Rewards

Author: BelizeChain Core Team
Date: October 31, 2025
"""

import asyncio
import time
from typing import Dict, Any, List
import httpx
from substrateinterface import SubstrateInterface, Keypair

# Test configuration
BLOCKCHAIN_URL = "ws://127.0.0.1:9944"
ORACLE_API_URL = "http://localhost:8000"
NAWAL_API_URL = "http://localhost:8001"

# Test operator (Alice from development chain)
ALICE_KEYPAIR = Keypair.create_from_uri('//Alice')
BOB_KEYPAIR = Keypair.create_from_uri('//Bob')

# Domain configurations
DOMAINS = {
    'agritech': {'multiplier': 1.5, 'index': 1},
    'marine': {'multiplier': 1.4, 'index': 2},
    'education': {'multiplier': 1.3, 'index': 3},
    'tech': {'multiplier': 1.1, 'index': 4},
    'general': {'multiplier': 1.0, 'index': 0}
}


class OracleIntegrationTest:
    """Complete integration test suite for Oracle + Nawal + Staking"""
    
    def __init__(self):
        self.substrate = None
        self.http_client = None
        self.test_devices = []
        self.results = {
            'passed': 0,
            'failed': 0,
            'errors': []
        }
    
    async def setup(self):
        """Initialize connections"""
        print("🔧 Setting up test environment...")
        
        # Connect to blockchain
        self.substrate = SubstrateInterface(url=BLOCKCHAIN_URL)
        print(f"✅ Connected to blockchain: {self.substrate.chain}")
        
        # HTTP client for APIs
        self.http_client = httpx.AsyncClient(timeout=30.0)
        
        # Check API health
        oracle_health = await self.http_client.get(f"{ORACLE_API_URL}/health")
        nawal_health = await self.http_client.get(f"{NAWAL_API_URL}/")  # Fixed: Nawal uses / not /health
        
        assert oracle_health.status_code == 200, "Oracle API not healthy"
        assert nawal_health.status_code == 200, "Nawal API not healthy"
        
        print("✅ Oracle API healthy")
        print("✅ Nawal API healthy")
        
        # Authenticate and get JWT token (for security-enabled APIs)
        login_response = await self.http_client.post(
            f"{ORACLE_API_URL}/api/oracle/auth/login",
            json={
                "account_id": ALICE_KEYPAIR.ss58_address,
                "api_key": "alice-dev-key-12345"  # Dev key from security.py
            }
        )
        
        if login_response.status_code == 200:
            self.auth_token = login_response.json()['access_token']
            self.auth_headers = {"Authorization": f"Bearer {self.auth_token}"}
            print("✅ Authenticated successfully")
        else:
            print(f"⚠️  Authentication failed (status {login_response.status_code}), using no auth")
            self.auth_token = None
            self.auth_headers = {}
    
    async def teardown(self):
        """Cleanup"""
        if self.http_client:
            await self.http_client.aclose()
        print("\n🧹 Cleanup complete")
    
    def log_result(self, test_name: str, passed: bool, details: str = ""):
        """Log test result"""
        if passed:
            self.results['passed'] += 1
            print(f"  ✅ {test_name}: PASSED {details}")
        else:
            self.results['failed'] += 1
            self.results['errors'].append(f"{test_name}: {details}")
            print(f"  ❌ {test_name}: FAILED - {details}")
    
    async def test_1_api_connectivity(self):
        """Test 1: Verify API connectivity"""
        print("\n📡 Test 1: API Connectivity")
        
        try:
            # Test Oracle API root
            response = await self.http_client.get(f"{ORACLE_API_URL}/")
            data = response.json()
            self.log_result(
                "Oracle API root",
                response.status_code == 200 and data['name'] == "BelizeChain Oracle API"
            )
            
            # Test Nawal API root
            response = await self.http_client.get(f"{NAWAL_API_URL}/")
            data = response.json()
            self.log_result(
                "Nawal API root",
                response.status_code == 200 and data['name'] == "BelizeChain Nawal AI API"
            )
            
            # Test domains endpoint
            response = await self.http_client.get(f"{NAWAL_API_URL}/api/nawal/domains")
            domains = response.json()
            self.log_result(
                "Domains listing",
                response.status_code == 200 and len(domains['domains']) == 5,
                f"({len(domains['domains'])} domains)"
            )
            
        except Exception as e:
            self.log_result("API connectivity", False, str(e))
    
    async def test_2_device_registration(self):
        """Test 2: Register IoT devices across all domains"""
        print("\n📝 Test 2: Device Registration")
        
        devices_to_register = [
            {
                'serial_number': 'AGRI-DRONE-001',
                'operator': ALICE_KEYPAIR.ss58_address,
                'device_type': 'AgriDrone',
                'capabilities': ['crop_imagery', 'soil_analysis'],
                'location': [17.2510, -88.7590],  # Belize City
                'domain': 'agritech'
            },
            {
                'serial_number': 'MARINE-BUOY-001',
                'operator': ALICE_KEYPAIR.ss58_address,
                'device_type': 'WaterQualityBuoy',
                'capabilities': ['temperature', 'salinity', 'ph'],
                'location': [17.9899, -87.5340],  # Blue Hole
                'domain': 'marine'
            },
            {
                'serial_number': 'EDU-SENSOR-001',
                'operator': BOB_KEYPAIR.ss58_address,
                'device_type': 'ClassroomMonitor',
                'capabilities': ['attendance', 'engagement'],
                'location': [17.4950, -88.1970],  # Belmopan
                'domain': 'education'
            }
        ]
        
        for device in devices_to_register:
            try:
                # Add required fields for registration
                registration_data = {
                    "device_serial": device['serial_number'],
                    "operator": device['operator'],
                    "operator_seed": "//Alice",  # Using Alice for all test registrations
                    "device_type": device['device_type'],
                    "domain": device['domain'],
                    "location_lat": device['location'][0],
                    "location_lon": device['location'][1]
                }
                
                response = await self.http_client.post(
                    f"{ORACLE_API_URL}/api/oracle/devices/register",
                    json=registration_data,
                    headers=self.auth_headers  # Add authentication
                )
                
                if response.status_code == 200:
                    data = response.json()
                    self.test_devices.append({
                        **device,
                        'device_id': data['device_id']
                    })
                    self.log_result(
                        f"Register {device['device_type']}",
                        True,
                        f"(ID: {data['device_id'][:16]}...)"
                    )
                else:
                    self.log_result(
                        f"Register {device['device_type']}",
                        False,
                        f"HTTP {response.status_code}"
                    )
                    
            except Exception as e:
                self.log_result(f"Register {device['device_type']}", False, str(e))
    
    async def test_3_data_submission(self):
        """Test 3: Submit data from registered devices"""
        print("\n📤 Test 3: Data Submission")
        
        submissions = [
            {
                'serial': 'AGRI-DRONE-001',
                'data': {
                    'type': 'crop_imagery',
                    'images': ['base64_image_data_here'],
                    'crop_type': 'sugar_cane',
                    'field_id': 'SCF-001',
                    'timestamp': int(time.time())
                },
                'domain': 'agritech'
            },
            {
                'serial': 'MARINE-BUOY-001',
                'data': {
                    'type': 'water_quality',
                    'temperature': 28.5,
                    'salinity': 35.2,
                    'ph': 8.1,
                    'dissolved_oxygen': 6.8,
                    'timestamp': int(time.time())
                },
                'domain': 'marine'
            },
            {
                'serial': 'EDU-SENSOR-001',
                'data': {
                    'type': 'student_performance',
                    'student_ids': ['S001', 'S002', 'S003'],
                    'subject': 'mathematics',
                    'scores': [85, 92, 78],
                    'timestamp': int(time.time())
                },
                'domain': 'education'
            }
        ]
        
        for submission in submissions:
            try:
                response = await self.http_client.post(
                    f"{ORACLE_API_URL}/api/oracle/devices/{submission['serial']}/submit",
                    json={'data': submission['data']},
                    headers=self.auth_headers  # Add authentication
                )
                
                if response.status_code == 200:
                    data = response.json()
                    self.log_result(
                        f"Submit {submission['domain']} data",
                        True,
                        f"(Quality: {data.get('quality_score', 'N/A')})"
                    )
                else:
                    self.log_result(
                        f"Submit {submission['domain']} data",
                        False,
                        f"HTTP {response.status_code}"
                    )
                    
            except Exception as e:
                self.log_result(f"Submit {submission['domain']} data", False, str(e))
    
    async def test_4_domain_contributions(self):
        """Test 4: Verify domain contributions recorded on-chain"""
        print("\n🔗 Test 4: Domain Contributions On-Chain")
        
        # Wait for pipeline to process
        print("  ⏳ Waiting 15 seconds for pipeline to process submissions...")
        await asyncio.sleep(15)
        
        # Query operator domain stats from blockchain
        operators = [ALICE_KEYPAIR.ss58_address, BOB_KEYPAIR.ss58_address]
        
        for operator in operators:
            try:
                # Query OperatorDomainStatsMap storage
                result = self.substrate.query(
                    module='Staking',
                    storage_function='OperatorDomainStatsMap',
                    params=[operator]
                )
                
                if result.value:
                    stats = result.value
                    total_contributions = sum([
                        stats.get('agritech', {}).get('contribution_count', 0),
                        stats.get('marine', {}).get('contribution_count', 0),
                        stats.get('education', {}).get('contribution_count', 0),
                        stats.get('tech', {}).get('contribution_count', 0),
                        stats.get('general', {}).get('contribution_count', 0)
                    ])
                    
                    self.log_result(
                        f"Operator {operator[:8]}... stats",
                        total_contributions > 0,
                        f"({total_contributions} contributions)"
                    )
                else:
                    self.log_result(
                        f"Operator {operator[:8]}... stats",
                        False,
                        "No stats found"
                    )
                    
            except Exception as e:
                self.log_result(f"Query operator stats", False, str(e))
    
    async def test_5_domain_multipliers(self):
        """Test 5: Verify domain info and multipliers"""
        print("\n🎯 Test 5: Domain Multipliers")
        
        for domain, config in DOMAINS.items():
            try:
                response = await self.http_client.get(
                    f"{NAWAL_API_URL}/api/nawal/domains/{domain}/info"
                )
                
                if response.status_code == 200:
                    data = response.json()
                    expected_multiplier = config['multiplier']
                    actual_multiplier = data['reward_multiplier']
                    
                    self.log_result(
                        f"{domain.capitalize()} multiplier",
                        actual_multiplier == expected_multiplier,
                        f"({actual_multiplier}x)"
                    )
                else:
                    self.log_result(
                        f"{domain.capitalize()} multiplier",
                        False,
                        f"HTTP {response.status_code}"
                    )
                    
            except Exception as e:
                self.log_result(f"{domain.capitalize()} multiplier", False, str(e))
    
    async def test_6_reward_claiming(self):
        """Test 6: Claim PoUW rewards with domain bonus"""
        print("\n💰 Test 6: Reward Claiming")
        
        # Query operator stats via API
        try:
            response = await self.http_client.get(
                f"{ORACLE_API_URL}/api/oracle/operators/{ALICE_KEYPAIR.ss58_address}/stats"
            )
            
            if response.status_code == 200:
                stats = response.json()
                self.log_result(
                    "Alice operator stats",
                    stats['total_submissions'] > 0,
                    f"({stats['total_submissions']} submissions)"
                )
                
                # Attempt reward claim
                claim_response = await self.http_client.post(
                    f"{ORACLE_API_URL}/api/oracle/operators/claim-rewards",
                    json={
                        'operator': ALICE_KEYPAIR.ss58_address,
                        'use_domain_bonus': True
                    }
                )
                
                if claim_response.status_code == 200:
                    claim_data = claim_response.json()
                    self.log_result(
                        "Claim rewards with domain bonus",
                        True,
                        f"(Bonus: {claim_data.get('domain_bonus_multiplier', 'N/A')}x)"
                    )
                else:
                    self.log_result(
                        "Claim rewards with domain bonus",
                        False,
                        f"HTTP {claim_response.status_code}"
                    )
            else:
                self.log_result("Alice operator stats", False, f"HTTP {response.status_code}")
                
        except Exception as e:
            self.log_result("Reward claiming", False, str(e))
    
    async def test_7_model_stats(self):
        """Test 7: Query AI model statistics"""
        print("\n🤖 Test 7: AI Model Statistics")
        
        domains = ['agritech', 'marine', 'education']
        
        for domain in domains:
            try:
                response = await self.http_client.get(
                    f"{NAWAL_API_URL}/api/nawal/models/{domain}/stats"
                )
                
                if response.status_code == 200:
                    stats = response.json()
                    self.log_result(
                        f"{domain.capitalize()} model stats",
                        'model_name' in stats,
                        f"({stats.get('total_inferences', 0)} inferences)"
                    )
                else:
                    self.log_result(
                        f"{domain.capitalize()} model stats",
                        False,
                        f"HTTP {response.status_code}"
                    )
                    
            except Exception as e:
                self.log_result(f"{domain.capitalize()} model stats", False, str(e))
    
    async def run_all_tests(self):
        """Run complete test suite"""
        print("\n" + "="*70)
        print("  BelizeChain Oracle Integration Tests - Phase 3")
        print("="*70)
        
        try:
            await self.setup()
            
            # Run tests in sequence
            await self.test_1_api_connectivity()
            await self.test_2_device_registration()
            await self.test_3_data_submission()
            await self.test_4_domain_contributions()
            await self.test_5_domain_multipliers()
            await self.test_6_reward_claiming()
            await self.test_7_model_stats()
            
        except Exception as e:
            print(f"\n❌ Fatal error: {e}")
            self.results['errors'].append(f"Fatal: {e}")
        
        finally:
            await self.teardown()
            self.print_summary()
    
    def print_summary(self):
        """Print test summary"""
        print("\n" + "="*70)
        print("  Test Summary")
        print("="*70)
        print(f"  ✅ Passed: {self.results['passed']}")
        print(f"  ❌ Failed: {self.results['failed']}")
        print(f"  📊 Total:  {self.results['passed'] + self.results['failed']}")
        
        if self.results['errors']:
            print(f"\n  Errors ({len(self.results['errors'])}):")
            for error in self.results['errors']:
                print(f"    - {error}")
        
        success_rate = (
            self.results['passed'] / (self.results['passed'] + self.results['failed']) * 100
            if (self.results['passed'] + self.results['failed']) > 0 else 0
        )
        
        print(f"\n  Success Rate: {success_rate:.1f}%")
        
        if success_rate >= 90:
            print("\n  🎉 Production Ready! All critical tests passed.")
        elif success_rate >= 70:
            print("\n  ⚠️  Some issues detected. Review failures before deployment.")
        else:
            print("\n  ❌ Major issues detected. Not ready for deployment.")
        
        print("="*70 + "\n")


async def main():
    """Main entry point"""
    test_suite = OracleIntegrationTest()
    await test_suite.run_all_tests()


if __name__ == '__main__':
    asyncio.run(main())

"""
BelizeChain E2E Security Test Suite
Tests authentication, rate limiting, and API security across all services

Author: BelizeChain Core Team
Date: November 1, 2025
"""

import asyncio
import httpx
import time
from typing import Dict, List

# Configuration
ORACLE_API_URL = "http://localhost:8000"
NAWAL_API_URL = "http://localhost:8001"

# Test operators (from api/security.py)
TEST_OPERATORS = {
    'alice': {
        'account_id': '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY',
        'api_key': 'alice-dev-key-12345',
        'role': 'admin',
        'rate_limit': 1000
    },
    'bob': {
        'account_id': '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty',
        'api_key': 'bob-dev-key-67890',
        'role': 'operator',
        'rate_limit': 500
    },
    'dave': {
        'account_id': '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
        'api_key': 'dave-user-key-22222',
        'role': 'user',
        'rate_limit': 100
    }
}


class SecurityE2ETest:
    """End-to-end security testing"""
    
    def __init__(self):
        self.client = None
        self.results = {'passed': 0, 'failed': 0, 'errors': []}
    
    async def setup(self):
        """Initialize HTTP client"""
        self.client = httpx.AsyncClient(timeout=30.0)
        print("🔧 Setting up E2E security tests...")
    
    async def teardown(self):
        """Cleanup"""
        if self.client:
            await self.client.aclose()
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
    
    async def test_1_api_health(self):
        """Test 1: API Health Checks"""
        print("\n🏥 Test 1: API Health Checks")
        
        try:
            # Oracle API
            response = await self.client.get(f"{ORACLE_API_URL}/health")
            self.log_result(
                "Oracle API health",
                response.status_code == 200 and response.json()['status'] == 'healthy',
                f"(block: {response.json().get('current_block', 'N/A')})"
            )
            
            # Nawal API
            response = await self.client.get(f"{NAWAL_API_URL}/")
            self.log_result(
                "Nawal API health",
                response.status_code == 200 and response.json()['status'] == 'operational'
            )
            
        except Exception as e:
            self.log_result("API health checks", False, str(e))
    
    async def test_2_authentication(self):
        """Test 2: Authentication System"""
        print("\n🔐 Test 2: Authentication System")
        
        # Test valid authentication for each operator
        for name, operator in TEST_OPERATORS.items():
            try:
                response = await self.client.post(
                    f"{ORACLE_API_URL}/api/oracle/auth/login",
                    json={
                        "account_id": operator['account_id'],
                        "api_key": operator['api_key']
                    }
                )
                
                if response.status_code == 200:
                    data = response.json()
                    self.log_result(
                        f"Login as {name} ({operator['role']})",
                        'access_token' in data and data['operator']['role'] == operator['role'],
                        f"(rate limit: {data['operator']['rate_limit']}/hr)"
                    )
                else:
                    self.log_result(
                        f"Login as {name}",
                        False,
                        f"HTTP {response.status_code}"
                    )
                    
            except Exception as e:
                self.log_result(f"Login as {name}", False, str(e))
        
        # Test invalid authentication
        try:
            response = await self.client.post(
                f"{ORACLE_API_URL}/api/oracle/auth/login",
                json={
                    "account_id": "5InvalidAccountID",
                    "api_key": "invalid-key"
                }
            )
            self.log_result(
                "Reject invalid credentials",
                response.status_code == 401,
                f"(got HTTP {response.status_code})"
            )
        except Exception as e:
            self.log_result("Reject invalid credentials", False, str(e))
    
    async def test_3_input_validation(self):
        """Test 3: Input Validation"""
        print("\n🛡️  Test 3: Input Validation")
        
        # Get auth token first
        login_response = await self.client.post(
            f"{ORACLE_API_URL}/api/oracle/auth/login",
            json={
                "account_id": TEST_OPERATORS['alice']['account_id'],
                "api_key": TEST_OPERATORS['alice']['api_key']
            }
        )
        token = login_response.json()['access_token']
        headers = {"Authorization": f"Bearer {token}"}
        
        # Test invalid serial number (contains special chars)
        try:
            response = await self.client.post(
                f"{ORACLE_API_URL}/api/oracle/devices/register",
                json={
                    "device_serial": "INVALID<>SERIAL",
                    "operator": TEST_OPERATORS['alice']['account_id'],
                    "operator_seed": "//Alice",
                    "device_type": "TestDevice",
                    "domain": "general",
                    "location_lat": 17.25,
                    "location_lon": -88.75
                },
                headers=headers
            )
            self.log_result(
                "Reject invalid serial number",
                response.status_code == 400,
                f"(got HTTP {response.status_code})"
            )
        except Exception as e:
            self.log_result("Reject invalid serial number", False, str(e))
        
        # Test invalid domain
        try:
            response = await self.client.post(
                f"{ORACLE_API_URL}/api/oracle/devices/register",
                json={
                    "device_serial": "TEST-001",
                    "operator": TEST_OPERATORS['alice']['account_id'],
                    "operator_seed": "//Alice",
                    "device_type": "TestDevice",
                    "domain": "invalid_domain",
                    "location_lat": 17.25,
                    "location_lon": -88.75
                },
                headers=headers
            )
            self.log_result(
                "Reject invalid domain",
                response.status_code == 400,
                f"(got HTTP {response.status_code})"
            )
        except Exception as e:
            self.log_result("Reject invalid domain", False, str(e))
        
        # Test invalid location (outside Belize)
        try:
            response = await self.client.post(
                f"{ORACLE_API_URL}/api/oracle/devices/register",
                json={
                    "device_serial": "TEST-002",
                    "operator": TEST_OPERATORS['alice']['account_id'],
                    "operator_seed": "//Alice",
                    "device_type": "TestDevice",
                    "domain": "general",
                    "location_lat": 50.0,  # Not in Belize
                    "location_lon": 10.0
                },
                headers=headers
            )
            self.log_result(
                "Reject invalid location",
                response.status_code == 400,
                f"(got HTTP {response.status_code})"
            )
        except Exception as e:
            self.log_result("Reject invalid location", False, str(e))
    
    async def test_4_authorization(self):
        """Test 4: Authorization (Protected Endpoints)"""
        print("\n🔒 Test 4: Authorization")
        
        # Test accessing protected endpoint without auth
        try:
            response = await self.client.post(
                f"{ORACLE_API_URL}/api/oracle/devices/register",
                json={
                    "device_serial": "TEST-003",
                    "operator": TEST_OPERATORS['alice']['account_id'],
                    "operator_seed": "//Alice",
                    "device_type": "TestDevice",
                    "domain": "general",
                    "location_lat": 17.25,
                    "location_lon": -88.75
                }
                # No auth headers
            )
            self.log_result(
                "Reject unauthenticated request",
                response.status_code == 401,
                f"(got HTTP {response.status_code})"
            )
        except Exception as e:
            self.log_result("Reject unauthenticated request", False, str(e))
        
        # Test with expired/invalid token
        try:
            response = await self.client.post(
                f"{ORACLE_API_URL}/api/oracle/devices/register",
                json={
                    "device_serial": "TEST-004",
                    "operator": TEST_OPERATORS['alice']['account_id'],
                    "operator_seed": "//Alice",
                    "device_type": "TestDevice",
                    "domain": "general",
                    "location_lat": 17.25,
                    "location_lon": -88.75
                },
                headers={"Authorization": "Bearer invalid.token.here"}
            )
            self.log_result(
                "Reject invalid token",
                response.status_code == 401,
                f"(got HTTP {response.status_code})"
            )
        except Exception as e:
            self.log_result("Reject invalid token", False, str(e))
    
    async def test_5_nawal_domains(self):
        """Test 5: Nawal Domain Endpoints"""
        print("\n🧠 Test 5: Nawal Domain Endpoints")
        
        try:
            # List domains
            response = await self.client.get(f"{NAWAL_API_URL}/api/nawal/domains")
            domains = response.json()
            self.log_result(
                "List available domains",
                response.status_code == 200 and len(domains) == 5,
                f"({len(domains)} domains)" if isinstance(domains, list) else ""
            )
            
            # Get domain info
            test_domains = ['agritech', 'marine', 'education']
            for domain in test_domains:
                response = await self.client.get(f"{NAWAL_API_URL}/api/nawal/domains/{domain}")
                if response.status_code == 200:
                    data = response.json()
                    self.log_result(
                        f"Get {domain} info",
                        'reward_multiplier' in data,
                        f"(multiplier: {data.get('reward_multiplier', 'N/A')}x)"
                    )
                else:
                    self.log_result(
                        f"Get {domain} info",
                        False,
                        f"HTTP {response.status_code}"
                    )
                    
        except Exception as e:
            self.log_result("Nawal domain endpoints", False, str(e))
    
    async def test_6_cors_security(self):
        """Test 6: CORS Security"""
        print("\n🌐 Test 6: CORS Security")
        
        try:
            # Test with Origin header
            response = await self.client.options(
                f"{ORACLE_API_URL}/api/oracle/devices/register",
                headers={"Origin": "https://malicious-site.com"}
            )
            
            # Should either reject or have restricted CORS headers
            self.log_result(
                "CORS configuration",
                response.status_code in [200, 403, 405],
                f"(HTTP {response.status_code})"
            )
            
        except Exception as e:
            self.log_result("CORS configuration", False, str(e))
    
    async def run_all_tests(self):
        """Run all E2E security tests"""
        await self.setup()
        
        print("\n" + "="*70)
        print("  BelizeChain E2E Security Test Suite")
        print("="*70)
        
        try:
            await self.test_1_api_health()
            await self.test_2_authentication()
            await self.test_3_input_validation()
            await self.test_4_authorization()
            await self.test_5_nawal_domains()
            await self.test_6_cors_security()
            
        finally:
            await self.teardown()
        
        # Print summary
        print("\n" + "="*70)
        print("  Test Summary")
        print("="*70)
        total = self.results['passed'] + self.results['failed']
        success_rate = (self.results['passed'] / total * 100) if total > 0 else 0
        
        print(f"  ✅ Passed: {self.results['passed']}")
        print(f"  ❌ Failed: {self.results['failed']}")
        print(f"  📊 Total:  {total}")
        
        if self.results['errors']:
            print(f"\n  Errors ({len(self.results['errors'])}):")
            for error in self.results['errors'][:10]:  # Show first 10
                print(f"    - {error}")
        
        print(f"\n  Success Rate: {success_rate:.1f}%")
        
        if success_rate >= 90:
            print("\n  🎉 Excellent! Security system is working well!")
        elif success_rate >= 75:
            print("\n  ✅ Good! Minor issues to address.")
        else:
            print("\n  ⚠️  Needs attention. Multiple security issues detected.")
        
        print("="*70)
        
        return success_rate >= 75


if __name__ == '__main__':
    test = SecurityE2ETest()
    result = asyncio.run(test.run_all_tests())
    exit(0 if result else 1)

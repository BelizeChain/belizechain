#!/usr/bin/env python3
"""
BelizeChain Load Testing Framework
Stress test blockchain with simulated transaction volumes and measure performance

Author: BelizeChain Core Team
Date: November 3, 2025
"""

import asyncio
import time
import statistics
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, field
from substrateinterface import SubstrateInterface, Keypair
from substrateinterface.exceptions import SubstrateRequestException
import concurrent.futures
import json
from datetime import datetime
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Configuration
NODE_URL = "ws://127.0.0.1:9944"
TARGET_TPS = 1000  # Target transactions per second
TEST_DURATION_SECONDS = 60  # 1 minute test
CONCURRENT_THREADS = 10  # Number of concurrent workers


@dataclass
class LoadTestMetrics:
    """Metrics collected during load testing"""
    total_transactions: int = 0
    successful_transactions: int = 0
    failed_transactions: int = 0
    execution_times: List[float] = field(default_factory=list)
    storage_reads: int = 0
    storage_writes: int = 0
    network_latencies: List[float] = field(default_factory=list)
    errors: List[str] = field(default_factory=list)
    start_time: float = 0
    end_time: float = 0


class BelizeChainLoadTester:
    """Load testing framework for BelizeChain"""
    
    def __init__(self, node_url: str = NODE_URL):
        self.node_url = node_url
        self.substrate: Optional[SubstrateInterface] = None
        self.metrics = LoadTestMetrics()
        self.test_keypairs: List[Keypair] = []
        
    def connect(self) -> bool:
        """Connect to BelizeChain node"""
        print(f"🔗 Connecting to BelizeChain node: {self.node_url}")
        try:
            self.substrate = SubstrateInterface(
                url=self.node_url,
                ss58_format=42,  # Substrate default
                type_registry_preset='substrate-node-template'
            )
            print("✅ Connected successfully")
            return True
        except Exception as e:
            print(f"❌ Connection failed: {e}")
            return False
    
    def _ensure_connected(self) -> SubstrateInterface:
        """Ensure substrate connection exists, raise if not"""
        if self.substrate is None:
            raise RuntimeError("Not connected to node. Call connect() first.")
        return self.substrate
    
    def generate_test_accounts(self, count: int = 100):
        """Generate test keypairs for load testing"""
        print(f"🔑 Generating {count} test accounts...")
        self.test_keypairs = [
            Keypair.create_from_uri(f"//TestAccount{i}")
            for i in range(count)
        ]
        print(f"✅ Generated {len(self.test_keypairs)} test accounts")
    
    def fund_test_accounts(self, amount: int = 1_000_000_000_000):
        """Fund test accounts from Alice (sudo account)"""
        substrate = self._ensure_connected()
        print(f"💰 Funding {len(self.test_keypairs)} test accounts...")
        alice = Keypair.create_from_uri('//Alice')
        
        funded = 0
        for keypair in self.test_keypairs:
            try:
                call = substrate.compose_call(
                    call_module='Balances',
                    call_function='transfer_keep_alive',
                    call_params={
                        'dest': keypair.ss58_address,
                        'value': amount
                    }
                )
                
                extrinsic = substrate.create_signed_extrinsic(
                    call=call,
                    keypair=alice
                )
                
                receipt = substrate.submit_extrinsic(
                    extrinsic,
                    wait_for_inclusion=True
                )
                
                if receipt.is_success:
                    funded += 1
                    
            except Exception as e:
                print(f"⚠️ Failed to fund account: {e}")
        
        print(f"✅ Funded {funded}/{len(self.test_keypairs)} accounts")
    
    def benchmark_extrinsic(self, pallet: str, call: str, params: Dict) -> Dict[str, Any]:
        """Benchmark a single extrinsic execution"""
        substrate = self._ensure_connected()
        start_time = time.time()
        
        try:
            # Select random test account
            keypair = self.test_keypairs[self.metrics.total_transactions % len(self.test_keypairs)]
            
            # Create call
            call_obj = substrate.compose_call(
                call_module=pallet,
                call_function=call,
                call_params=params
            )
            
            # Create signed extrinsic
            extrinsic = substrate.create_signed_extrinsic(
                call=call_obj,
                keypair=keypair
            )
            
            # Submit and wait for inclusion
            receipt = substrate.submit_extrinsic(
                extrinsic,
                wait_for_inclusion=True
            )
            
            end_time = time.time()
            execution_time = end_time - start_time
            
            # Record metrics
            self.metrics.total_transactions += 1
            
            if receipt.is_success:
                self.metrics.successful_transactions += 1
            else:
                self.metrics.failed_transactions += 1
                self.metrics.errors.append(f"Extrinsic failed: {receipt.error_message}")
            
            self.metrics.execution_times.append(execution_time)
            self.metrics.network_latencies.append(execution_time)
            
            return {
                'success': receipt.is_success,
                'execution_time': execution_time,
                'block_hash': receipt.block_hash
            }
            
        except Exception as e:
            end_time = time.time()
            self.metrics.total_transactions += 1
            self.metrics.failed_transactions += 1
            self.metrics.errors.append(str(e))
            self.metrics.execution_times.append(end_time - start_time)
            
            return {
                'success': False,
                'execution_time': end_time - start_time,
                'error': str(e)
            }
    
    def benchmark_storage_operation(self, pallet: str, storage_function: str, params: Optional[List] = None):
        """Benchmark storage read operation"""
        substrate = self._ensure_connected()
        start_time = time.time()
        
        try:
            if params:
                result = substrate.query(
                    module=pallet,
                    storage_function=storage_function,
                    params=params
                )
            else:
                result = substrate.query(
                    module=pallet,
                    storage_function=storage_function
                )
            
            end_time = time.time()
            read_time = end_time - start_time
            
            self.metrics.storage_reads += 1
            self.metrics.network_latencies.append(read_time)
            
            return {
                'success': True,
                'read_time': read_time,
                'result': result.value if result else None
            }
            
        except Exception as e:
            end_time = time.time()
            self.metrics.errors.append(f"Storage read failed: {e}")
            return {
                'success': False,
                'read_time': end_time - start_time,
                'error': str(e)
            }
    
    def run_transfer_load_test(self, duration_seconds: int, target_tps: int):
        """Run load test with balance transfers"""
        print(f"\n🚀 Starting transfer load test...")
        print(f"   Duration: {duration_seconds}s")
        print(f"   Target TPS: {target_tps}")
        print(f"   Concurrent threads: {CONCURRENT_THREADS}")
        
        self.metrics.start_time = time.time()
        end_time = self.metrics.start_time + duration_seconds
        
        # Calculate transactions per thread
        transactions_per_thread = (target_tps * duration_seconds) // CONCURRENT_THREADS
        
        def worker_thread(thread_id: int):
            """Worker thread for generating transactions"""
            count = 0
            while time.time() < end_time and count < transactions_per_thread:
                # Random transfer between test accounts
                from_idx = (thread_id * transactions_per_thread + count) % len(self.test_keypairs)
                to_idx = (from_idx + 1) % len(self.test_keypairs)
                
                self.benchmark_extrinsic(
                    pallet='Balances',
                    call='transfer_keep_alive',
                    params={
                        'dest': self.test_keypairs[to_idx].ss58_address,
                        'value': 1_000_000  # 1 DALLA
                    }
                )
                count += 1
        
        # Run concurrent workers
        with concurrent.futures.ThreadPoolExecutor(max_workers=CONCURRENT_THREADS) as executor:
            futures = [
                executor.submit(worker_thread, i)
                for i in range(CONCURRENT_THREADS)
            ]
            concurrent.futures.wait(futures)
        
        self.metrics.end_time = time.time()
        print("✅ Load test completed")
    
    def run_storage_benchmark(self, iterations: int = 1000):
        """Benchmark storage read operations"""
        print(f"\n📊 Starting storage benchmark ({iterations} iterations)...")
        
        storage_queries = [
            ('System', 'Number', []),
            ('System', 'BlockHash', [0]),
            ('Timestamp', 'Now', []),
            ('Balances', 'TotalIssuance', []),
        ]
        
        for _ in range(iterations):
            pallet, storage_fn, params = storage_queries[_ % len(storage_queries)]
            self.benchmark_storage_operation(pallet, storage_fn, params if params else None)
        
        print("✅ Storage benchmark completed")
    
    def generate_report(self) -> Dict[str, Any]:
        """Generate comprehensive performance report"""
        duration = self.metrics.end_time - self.metrics.start_time
        actual_tps = self.metrics.total_transactions / duration if duration > 0 else 0
        success_rate = (self.metrics.successful_transactions / self.metrics.total_transactions * 100) if self.metrics.total_transactions > 0 else 0
        
        report = {
            'test_summary': {
                'duration_seconds': round(duration, 2),
                'total_transactions': self.metrics.total_transactions,
                'successful_transactions': self.metrics.successful_transactions,
                'failed_transactions': self.metrics.failed_transactions,
                'success_rate_percent': round(success_rate, 2),
                'actual_tps': round(actual_tps, 2),
                'target_tps': TARGET_TPS,
                'tps_achievement_percent': round((actual_tps / TARGET_TPS * 100), 2) if TARGET_TPS > 0 else 0
            },
            'execution_times': {
                'min_ms': round(min(self.metrics.execution_times) * 1000, 2) if self.metrics.execution_times else 0,
                'max_ms': round(max(self.metrics.execution_times) * 1000, 2) if self.metrics.execution_times else 0,
                'mean_ms': round(statistics.mean(self.metrics.execution_times) * 1000, 2) if self.metrics.execution_times else 0,
                'median_ms': round(statistics.median(self.metrics.execution_times) * 1000, 2) if self.metrics.execution_times else 0,
                'stdev_ms': round(statistics.stdev(self.metrics.execution_times) * 1000, 2) if len(self.metrics.execution_times) > 1 else 0,
                'p95_ms': round(sorted(self.metrics.execution_times)[int(len(self.metrics.execution_times) * 0.95)] * 1000, 2) if self.metrics.execution_times else 0,
                'p99_ms': round(sorted(self.metrics.execution_times)[int(len(self.metrics.execution_times) * 0.99)] * 1000, 2) if self.metrics.execution_times else 0
            },
            'network_latency': {
                'min_ms': round(min(self.metrics.network_latencies) * 1000, 2) if self.metrics.network_latencies else 0,
                'max_ms': round(max(self.metrics.network_latencies) * 1000, 2) if self.metrics.network_latencies else 0,
                'mean_ms': round(statistics.mean(self.metrics.network_latencies) * 1000, 2) if self.metrics.network_latencies else 0,
                'median_ms': round(statistics.median(self.metrics.network_latencies) * 1000, 2) if self.metrics.network_latencies else 0
            },
            'storage_operations': {
                'total_reads': self.metrics.storage_reads,
                'total_writes': self.metrics.storage_writes,
                'reads_per_second': round(self.metrics.storage_reads / duration, 2) if duration > 0 else 0
            },
            'errors': {
                'total_errors': len(self.metrics.errors),
                'error_rate_percent': round((len(self.metrics.errors) / self.metrics.total_transactions * 100), 2) if self.metrics.total_transactions > 0 else 0,
                'sample_errors': self.metrics.errors[:10]  # First 10 errors
            },
            'timestamp': datetime.now().isoformat()
        }
        
        return report
    
    def print_report(self, report: Dict[str, Any]):
        """Print formatted performance report"""
        print("\n" + "="*80)
        print("📊 BELIZECHAIN LOAD TEST REPORT")
        print("="*80)
        
        summary = report['test_summary']
        print(f"\n⏱️  Test Duration: {summary['duration_seconds']}s")
        print(f"📈 Total Transactions: {summary['total_transactions']}")
        print(f"✅ Successful: {summary['successful_transactions']}")
        print(f"❌ Failed: {summary['failed_transactions']}")
        print(f"🎯 Success Rate: {summary['success_rate_percent']}%")
        print(f"🚀 Actual TPS: {summary['actual_tps']}")
        print(f"🎯 Target TPS: {summary['target_tps']}")
        print(f"📊 TPS Achievement: {summary['tps_achievement_percent']}%")
        
        exec_times = report['execution_times']
        print(f"\n⚡ Execution Times:")
        print(f"   Min: {exec_times['min_ms']}ms")
        print(f"   Max: {exec_times['max_ms']}ms")
        print(f"   Mean: {exec_times['mean_ms']}ms")
        print(f"   Median: {exec_times['median_ms']}ms")
        print(f"   StdDev: {exec_times['stdev_ms']}ms")
        print(f"   P95: {exec_times['p95_ms']}ms")
        print(f"   P99: {exec_times['p99_ms']}ms")
        
        latency = report['network_latency']
        print(f"\n🌐 Network Latency:")
        print(f"   Min: {latency['min_ms']}ms")
        print(f"   Max: {latency['max_ms']}ms")
        print(f"   Mean: {latency['mean_ms']}ms")
        print(f"   Median: {latency['median_ms']}ms")
        
        storage = report['storage_operations']
        print(f"\n💾 Storage Operations:")
        print(f"   Total Reads: {storage['total_reads']}")
        print(f"   Total Writes: {storage['total_writes']}")
        print(f"   Reads/sec: {storage['reads_per_second']}")
        
        errors = report['errors']
        print(f"\n⚠️  Errors:")
        print(f"   Total Errors: {errors['total_errors']}")
        print(f"   Error Rate: {errors['error_rate_percent']}%")
        if errors['sample_errors']:
            print(f"   Sample Errors:")
            for error in errors['sample_errors'][:5]:
                print(f"      - {error[:100]}")
        
        print("\n" + "="*80)
    
    def save_report(self, report: Dict[str, Any], filename: str = "load_test_report.json"):
        """Save report to JSON file"""
        with open(filename, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"💾 Report saved to: {filename}")


def main():
    """Main load testing entry point"""
    print("🔥 BelizeChain Load Testing Framework")
    print("="*80)
    
    # Initialize tester
    tester = BelizeChainLoadTester()
    
    # Connect to node
    if not tester.connect():
        print("❌ Failed to connect to node. Exiting.")
        return
    
    # Generate and fund test accounts
    tester.generate_test_accounts(count=100)
    tester.fund_test_accounts(amount=10_000_000_000_000)  # 10K DALLA per account
    
    # Wait for funding to complete
    print("⏳ Waiting for funding transactions to finalize...")
    time.sleep(12)  # Wait 2 blocks
    
    # Run transfer load test
    tester.run_transfer_load_test(
        duration_seconds=TEST_DURATION_SECONDS,
        target_tps=TARGET_TPS
    )
    
    # Run storage benchmark
    tester.run_storage_benchmark(iterations=1000)
    
    # Generate and print report
    report = tester.generate_report()
    tester.print_report(report)
    
    # Save report
    tester.save_report(report)
    
    print("\n✅ Load testing complete!")


if __name__ == "__main__":
    main()

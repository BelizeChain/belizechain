#!/bin/bash
# Fix all enum type conversions in tests.rs

# Fix ActivityType function parameters (not already converted)
sed -i '/ActivityType::[A-Za-z]*$/s/ActivityType::\([A-Za-z]*\)$/ActivityType::\1.as_u8()/' tests.rs

# Fix EndorsementType function parameters (not already converted)  
sed -i '/EndorsementType::[A-Za-z]*$/s/EndorsementType::\([A-Za-z]*\)$/EndorsementType::\1.as_u8()/' tests.rs

# Fix CommunityProposalType function parameters (not already converted)
sed -i '/CommunityProposalType::[A-Za-z]*,$/s/CommunityProposalType::\([A-Za-z]*\),$/CommunityProposalType::\1.as_u8(),/' tests.rs

# Fix comparison lines - remove .as_u8() from comparisons with enum fields
sed -i 's/\(activity_type.*==.*ActivityType::[A-Za-z]*\)\.as_u8()/\1/g' tests.rs

echo "Applied all test fixes"

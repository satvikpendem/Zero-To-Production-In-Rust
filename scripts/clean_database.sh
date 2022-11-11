#!/bin/bash
# SQL to select all databases created by specific user
# -t only tuple
# -A output not unaligned
# -q quiet
# -X Don't run .psqlrc file
# -o output file
# -c SQL command

# Generate a random string
RAND=$RANDOM
TEMP=/tmp/database_$RAND.txt

echo -e "Beginning cleaning..."

PSQL="psql"
if [[ "$(uname)" == "Linux" ]]; then PSQL="sudo -u postgres psql"; else PSQL="psql"; fi

if [[ -z "$1" ]]; then
    echo -e "No database role specified. Exiting..."
    exit 1
fi

$PSQL -qtAX -o $TEMP -c "SELECT datname FROM pg_database JOIN pg_authid ON pg_database.datdba = pg_authid.oid WHERE rolname = '$1';"

# echo after sudo asks for password for cleaner output
echo -e "Storing results in\n\t$TEMP"
echo -e "Selecting databases with role $1..."

echo -e "Dropping..."
# Drop all databases from database.txt
while read database; do
    # Disconnect from database
    $PSQL -qtAX -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='$database';"

    # Drop database
    echo -e "\t $database..."

    USER=""
    if [[ "$(uname)" == "Linux" ]]; then USER="sudo -u postgres"; else USER=""; fi
    $USER dropdb $database
done <$TEMP

echo -e "Checking if file exists\n\t$TEMP..."
if [ -f "$TEMP" ]; then
    echo -e "Removing\n\t$TEMP..."
    sudo rm $TEMP
fi

echo -e "Done"

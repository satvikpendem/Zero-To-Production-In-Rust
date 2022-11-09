# SQL to select all databases created by specific user
# -t only tuple
# -A output not unaligned
# -q quiet
# -X Don't run .psqlrc file
# -o output file
# -c SQL command

echo "Beginning cleaning..."

sudo -u postgres psql -qtAX -o /tmp/database.txt -c "SELECT datname FROM pg_database JOIN pg_authid ON pg_database.datdba = pg_authid.oid WHERE rolname = '$1';"
# echo after sudo asks for password for cleaner output
echo "Selecting databases with role $($1)..."

echo "Dropping..."
# Drop all databases from database.txt
while read database; do
    # Disconnect from database
    sudo -u postgres psql -qtAX -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='$database';"

    # Drop database
    echo "\t $database..."
    sudo -u postgres dropdb $database
done </tmp/database.txt

echo "Removing..."
sudo rm /tmp/database.txt

echo "Done"

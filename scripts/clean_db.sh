# SQL to select all databases created by specific user
# -t only tuple
# -A output not unaligned
# -q quiet
# -X Don't run .psqlrc file
# -o output file
# -c SQL command

sudo -u postgres psql -qtAX -o /tmp/db.txt -c "SELECT datname FROM pg_database JOIN pg_authid ON pg_database.datdba = pg_authid.oid WHERE rolname = 'newsletter';"
# echo after sudo asks for password for cleaner output
echo "Selecting..."

echo "Dropping..."
# Drop all databases from db.txt
while read db; do
    # Disconnect from database
    sudo -u postgres psql -qtAX -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='$db';"
    # Drop database
    
    echo "\t $db..."
    sudo -u postgres dropdb $db
done < /tmp/db.txt

echo "Removing..."
sudo rm /tmp/db.txt

echo "Done"
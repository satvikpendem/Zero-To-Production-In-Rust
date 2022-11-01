# SQL to select all databases created by specific user
sudo -u postgres psql -o /tmp/db.txt -c "SELECT datname FROM pg_database JOIN pg_authid ON pg_database.datdba = pg_authid.oid WHERE rolname = 'newsletter';"

# Drop all databases from db.txt
while read p; do
    # Disconnect from database
    sudo -u postgres psql -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname='$p';"
    # Drop database
    sudo -u postgres dropdb $p
done < /tmp/db.txt

sudo rm /tmp/db.txt
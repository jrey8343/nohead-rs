-- passwords are all hashed versions of 'testing12345'
INSERT INTO users (email, password_hash) VALUES (
    'hownow@browncow.com',
    '$argon2id$v=19$m=19456,t=2,p=1$MYlgvlS1auvOAHtS5qnpVA$o4NMH0IGELKfiakQGCC+I8DRBBR+AAfhTC+65dBcbf8'
);


# Nutty BIP-353 Human Bitcoin Address API

Nutty provides a simple interface and DNS-based resolution for Bitcoin payment addresses (BIP-353). Use this API to register human-readable addresses (e.g., `user@nutty.cash`).

## 1. Address Resolution (DNS)

To resolve a Bitcoin human-readable address to a BIP-21 URI:

```bash
curl -H "accept: application/dns-json" \
     "https://cloudflare-dns.com/dns-query?name=user.user._bitcoin-payment.example.com&type=TXT"
```

## 2. Registering an Address

Endpoint: `POST /api/v1/address`

### Validation Rules
- **user_name**: Optional. If provided, must be **at least 4 characters** and contains only `a-z`, `A-Z`, `0-9`, `.`, `-`, or `_`.
- **domain**: Required. Must be a supported domain.
- **Payment Info**: **At least one** of `sp`, `lno`, or `creq` is required.
  - `sp`: Silent Payment address (starts with `sp1q`).
  - `lno`: BOLT 12 Offer (starts with `lno1`).
  - `creq`: Cashu payment request (NUT-26, starts with `creqb1`).

---

### Step 1: Request Registration
Submit the desired name and payment details.

```bash
curl -i -X POST https://example.com/api/v1/address \
     -H "Content-Type: application/json" \
     -d '{
       "user_name": "yourname",
       "domain": "example.com",
       "sp": "sp1q..."
     }'
```

**Response: `402 Payment Required`**
If the name is available but requires payment:
- **Header**: `X-Cashu` contains a NUT-18 payment request string.
- **Body**: JSON containing the price and accepted mints.
  ```json
  {
    "user_name": "yourname",
    "amount": 5000,
    "unit": "sat",
    "message": "Payment Required. See help link for instructions on how to purchase.",
    "accepted_mints": ["https://mint.host/"],
    "help": "https://example.com/api/v1/SKILL.md"
  }
  ```
*Note: If `user_name` was omitted in the request, the body will contain a server-generated random name.*

---

### Step 2: Submit Payment Token
The user must create a **Cashu Token (NUT-00)** that satisfies the payment request provided in Step 1. This token must:
1. Match the **amount** and **unit** (sats) specified.
2. Be issued by one of the **accepted_mints**.
3. Be included in the `X-Cashu` header of a second POST request.

```bash
curl -X POST https://example.com/api/v1/address \
     -H "Content-Type: application/json" \
     -H "X-Cashu: <cashu_token_v3>" \
     -d '{
       "user_name": "yourname",
       "domain": "example.com",
       "sp": "sp1q..."
     }'
```

**Response: `200 OK`**
```json
{
  "status": "active",
  "user_name": "yourname",
  "domain": "example.com",
  "bip353": "yourname@example.com"
}
```

---

### Common Error Codes
- `409 Conflict`: Username is already taken.
- `422 Unprocessable Entity`: Validation failed (e.g., name too short or invalid characters).
- `400 Bad Request`: Missing mandatory fields (like no payment address provided).
- `500 Internal Server Error`: Failed to create DNS record or redeem token.

## Payload Fields Summary
- `user_name`: (Optional) Desired name.
- `domain`: The domain name.
- `sp`: (Optional) Silent Payment address.
- `lno`: (Optional) BOLT 12 Offer.
- `creq`: (Optional) Cashu payment request (NUT-26).

INSERT INTO reservations (guest_name, party_size, reserved_at, table_number, tenant_id)
VALUES
  ('Alex', 2, '2026-01-17T18:30:00Z', 4, 'puppyrestaurant'),
  ('Jamie', 4, '2026-01-17T19:00:00Z', 8, 'puppyrestaurant'),
  ('Taylor', 3, '2026-01-17T19:30:00Z', 5, 'puppyrestaurant');

INSERT INTO diners (name, vip_level, tenant_id)
VALUES
  ('Alex', 'silver', 'puppyrestaurant'),
  ('Jamie', 'gold', 'puppyrestaurant'),
  ('Taylor', 'bronze', 'puppyrestaurant');

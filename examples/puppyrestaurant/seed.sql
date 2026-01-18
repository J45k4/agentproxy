INSERT INTO reservations (guest_name, party_size, reserved_at, table_number, status, tenant_id)
VALUES
  ('Alex', 2, '2026-01-17T18:30:00Z', 4, 'booked', 'puppyrestaurant'),
  ('Jamie', 4, '2026-01-17T19:00:00Z', 8, 'seated', 'puppyrestaurant'),
  ('Taylor', 3, '2026-01-17T19:30:00Z', 5, 'booked', 'puppyrestaurant');

INSERT INTO diners (name, vip_level, tenant_id)
VALUES
  ('Alex', 'silver', 'puppyrestaurant'),
  ('Jamie', 'gold', 'puppyrestaurant'),
  ('Taylor', 'bronze', 'puppyrestaurant');

INSERT INTO menu_items (name, category, price_cents, tenant_id)
VALUES
  ('Puppy Pasta', 'entree', 1599, 'puppyrestaurant'),
  ('Garden Salad', 'appetizer', 799, 'puppyrestaurant'),
  ('Sparkling Water', 'beverage', 399, 'puppyrestaurant');

INSERT INTO orders (reservation_id, status, total_cents, tenant_id)
VALUES
  (1, 'open', 2397, 'puppyrestaurant'),
  (2, 'paid', 1998, 'puppyrestaurant');

INSERT INTO order_items (order_id, menu_item_id, quantity, line_total_cents, tenant_id)
VALUES
  (1, 1, 1, 1599, 'puppyrestaurant'),
  (1, 2, 1, 798, 'puppyrestaurant'),
  (2, 1, 1, 1599, 'puppyrestaurant'),
  (2, 3, 1, 399, 'puppyrestaurant');

INSERT INTO carts (diner_id, status, subtotal_cents, tenant_id)
VALUES
  (1, 'active', 1998, 'puppyrestaurant'),
  (3, 'active', 1599, 'puppyrestaurant');

INSERT INTO cart_items (cart_id, menu_item_id, quantity, line_total_cents, tenant_id)
VALUES
  (1, 1, 1, 1599, 'puppyrestaurant'),
  (1, 3, 1, 399, 'puppyrestaurant'),
  (2, 1, 1, 1599, 'puppyrestaurant');

INSERT INTO payments (order_id, provider, status, amount_cents, created_at, tenant_id)
VALUES
  (2, 'stripe', 'captured', 1998, '2026-01-17T20:05:00Z', 'puppyrestaurant');

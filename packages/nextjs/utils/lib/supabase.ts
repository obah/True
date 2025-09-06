import { createClient } from "@supabase/supabase-js";

const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL;
const supabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY;

if (!supabaseUrl || !supabaseAnonKey) {
  throw new Error("Missing Supabase environment variables");
}

export const supabase = createClient(supabaseUrl, supabaseAnonKey);

// Database types
export interface NotificationData {
  id: string;
  user_address: string;
  type: "transfer_code_generated" | "transfer_code_revoked" | "ownership_claimed";
  title: string;
  message: string;
  item_id: string;
  item_name: string;
  from_address?: string;
  to_address?: string;
  ownership_code?: string;
  is_read: boolean;
  created_at: string;
}

export interface TransferCodeData {
  id: string;
  item_id: string;
  item_name: string;
  from_address: string;
  to_address: string;
  ownership_code: string;
  is_active: boolean;
  created_at: string;
  expires_at: string;
}

// Notification functions
export const createNotification = async (notification: Omit<NotificationData, "id" | "created_at" | "is_read">) => {
  const { data, error } = await supabase
    .from("notifications")
    .insert([
      {
        ...notification,
        is_read: false,
      },
    ])
    .select()
    .single();

  if (error) throw error;
  return data;
};

export const getNotifications = async (userAddress: string) => {
  const { data, error } = await supabase
    .from("notifications")
    .select("*")
    .eq("user_address", userAddress.toLowerCase())
    .order("created_at", { ascending: false });

  if (error) throw error;
  return data as NotificationData[];
};

export const markNotificationAsRead = async (notificationId: string) => {
  const { error } = await supabase.from("notifications").update({ is_read: true }).eq("id", notificationId);

  if (error) throw error;
};

export const markAllNotificationsAsRead = async (userAddress: string) => {
  const { error } = await supabase
    .from("notifications")
    .update({ is_read: true })
    .eq("user_address", userAddress.toLowerCase())
    .eq("is_read", false);

  if (error) throw error;
};

// Transfer code functions
export const saveTransferCode = async (transferCode: Omit<TransferCodeData, "id" | "created_at">) => {
  // First check if a transfer code already exists for this combination
  const { data: existing } = await supabase
    .from("transfer_codes")
    .select("*")
    .eq("ownership_code", transferCode.ownership_code)
    .single();

  if (existing) {
    // If code exists and is active, return the existing one
    if (existing.is_active) {
      return existing;
    } else {
      // If code exists but is inactive, reactivate it
      const { data, error } = await supabase
        .from("transfer_codes")
        .update({
          is_active: true,
          expires_at: transferCode.expires_at,
        })
        .eq("ownership_code", transferCode.ownership_code)
        .select()
        .single();

      if (error) throw error;
      return data;
    }
  }

  // If no existing code, create a new one
  const { data, error } = await supabase.from("transfer_codes").insert([transferCode]).select().single();

  if (error) throw error;
  return data;
};

export const getTransferCode = async (ownershipCode: string) => {
  const { data, error } = await supabase
    .from("transfer_codes")
    .select("*")
    .eq("ownership_code", ownershipCode)
    .eq("is_active", true)
    .single();

  if (error) throw error;
  return data as TransferCodeData;
};

export const revokeTransferCode = async (ownershipCode: string) => {
  const { error } = await supabase
    .from("transfer_codes")
    .update({ is_active: false })
    .eq("ownership_code", ownershipCode);

  if (error) throw error;
};

export const getActiveTransferCodes = async (fromAddress: string) => {
  const { data, error } = await supabase
    .from("transfer_codes")
    .select("*")
    .eq("from_address", fromAddress.toLowerCase())
    .eq("is_active", true)
    .order("created_at", { ascending: false });

  if (error) throw error;
  return data as TransferCodeData[];
};

// Real-time subscription for notifications
export const subscribeToNotifications = (userAddress: string, callback: (notification: NotificationData) => void) => {
  return supabase
    .channel("notifications")
    .on(
      "postgres_changes",
      {
        event: "INSERT",
        schema: "public",
        table: "notifications",
        filter: `user_address=eq.${userAddress.toLowerCase()}`,
      },
      payload => {
        callback(payload.new as NotificationData);
      },
    )
    .subscribe();
};

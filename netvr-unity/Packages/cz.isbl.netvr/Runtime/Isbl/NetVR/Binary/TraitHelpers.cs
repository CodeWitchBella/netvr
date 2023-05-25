using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {
    static class TraitHelpers {
        public static void serialize_map_u32_to_RemoteClientSnapshot(Serde.ValueDictionary<uint, RemoteClientSnapshot> value, Serde.ISerializer serializer) {
            serializer.serialize_len(value.Count);
            int[] offsets = new int[value.Count];
            int count = 0;
            foreach (KeyValuePair<uint, RemoteClientSnapshot> entry in value) {
                offsets[count++] = serializer.get_buffer_offset();
                serializer.serialize_u32(entry.Key);
                entry.Value.Serialize(serializer);
            }
            serializer.sort_map_entries(offsets);
        }

        public static Serde.ValueDictionary<uint, RemoteClientSnapshot> deserialize_map_u32_to_RemoteClientSnapshot(Serde.IDeserializer deserializer) {
            long length = deserializer.deserialize_len();
            var obj = new Dictionary<uint, RemoteClientSnapshot>();
            int previous_key_start = 0;
            int previous_key_end = 0;
            for (long i = 0; i < length; i++) {
                int key_start = deserializer.get_buffer_offset();
                var key = deserializer.deserialize_u32();
                int key_end = deserializer.get_buffer_offset();
                if (i > 0) {
                    deserializer.check_that_key_slices_are_increasing(
                        new Serde.Range(previous_key_start, previous_key_end),
                        new Serde.Range(key_start, key_end));
                }
                previous_key_start = key_start;
                previous_key_end = key_end;
                var value = RemoteClientSnapshot.Deserialize(deserializer);
                obj[key] = value;
            }
            return new Serde.ValueDictionary<uint, RemoteClientSnapshot>(obj);
        }

        public static void serialize_vector_Controller(Serde.ValueArray<Controller> value, Serde.ISerializer serializer) {
            serializer.serialize_len(value.Count);
            foreach (var item in value) {
                item.Serialize(serializer);
            }
        }

        public static Serde.ValueArray<Controller> deserialize_vector_Controller(Serde.IDeserializer deserializer) {
            long length = deserializer.deserialize_len();
            Controller[] obj = new Controller[length];
            for (int i = 0; i < length; i++) {
                obj[i] = Controller.Deserialize(deserializer);
            }
            return new Serde.ValueArray<Controller>(obj);
        }

        public static void serialize_vector_Pose(Serde.ValueArray<Pose> value, Serde.ISerializer serializer) {
            serializer.serialize_len(value.Count);
            foreach (var item in value) {
                item.Serialize(serializer);
            }
        }

        public static Serde.ValueArray<Pose> deserialize_vector_Pose(Serde.IDeserializer deserializer) {
            long length = deserializer.deserialize_len();
            Pose[] obj = new Pose[length];
            for (int i = 0; i < length; i++) {
                obj[i] = Pose.Deserialize(deserializer);
            }
            return new Serde.ValueArray<Pose>(obj);
        }

        public static void serialize_vector_RemoteAction(Serde.ValueArray<RemoteAction> value, Serde.ISerializer serializer) {
            serializer.serialize_len(value.Count);
            foreach (var item in value) {
                item.Serialize(serializer);
            }
        }

        public static Serde.ValueArray<RemoteAction> deserialize_vector_RemoteAction(Serde.IDeserializer deserializer) {
            long length = deserializer.deserialize_len();
            RemoteAction[] obj = new RemoteAction[length];
            for (int i = 0; i < length; i++) {
                obj[i] = RemoteAction.Deserialize(deserializer);
            }
            return new Serde.ValueArray<RemoteAction>(obj);
        }

        public static void serialize_vector_RemoteDevice(Serde.ValueArray<RemoteDevice> value, Serde.ISerializer serializer) {
            serializer.serialize_len(value.Count);
            foreach (var item in value) {
                item.Serialize(serializer);
            }
        }

        public static Serde.ValueArray<RemoteDevice> deserialize_vector_RemoteDevice(Serde.IDeserializer deserializer) {
            long length = deserializer.deserialize_len();
            RemoteDevice[] obj = new RemoteDevice[length];
            for (int i = 0; i < length; i++) {
                obj[i] = RemoteDevice.Deserialize(deserializer);
            }
            return new Serde.ValueArray<RemoteDevice>(obj);
        }

        public static void serialize_vector_RemoteInteractionProfile(Serde.ValueArray<RemoteInteractionProfile> value, Serde.ISerializer serializer) {
            serializer.serialize_len(value.Count);
            foreach (var item in value) {
                item.Serialize(serializer);
            }
        }

        public static Serde.ValueArray<RemoteInteractionProfile> deserialize_vector_RemoteInteractionProfile(Serde.IDeserializer deserializer) {
            long length = deserializer.deserialize_len();
            RemoteInteractionProfile[] obj = new RemoteInteractionProfile[length];
            for (int i = 0; i < length; i++) {
                obj[i] = RemoteInteractionProfile.Deserialize(deserializer);
            }
            return new Serde.ValueArray<RemoteInteractionProfile>(obj);
        }

        public static void serialize_vector_str(Serde.ValueArray<string> value, Serde.ISerializer serializer) {
            serializer.serialize_len(value.Count);
            foreach (var item in value) {
                serializer.serialize_str(item);
            }
        }

        public static Serde.ValueArray<string> deserialize_vector_str(Serde.IDeserializer deserializer) {
            long length = deserializer.deserialize_len();
            string[] obj = new string[length];
            for (int i = 0; i < length; i++) {
                obj[i] = deserializer.deserialize_str();
            }
            return new Serde.ValueArray<string>(obj);
        }

    }


} // end of namespace Isbl.NetVR.Binary

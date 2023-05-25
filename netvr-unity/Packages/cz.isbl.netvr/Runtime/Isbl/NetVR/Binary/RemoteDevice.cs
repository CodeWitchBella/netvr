using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class RemoteDevice: IEquatable<RemoteDevice>, ICloneable {
        public uint id;
        public Vec3 pos;
        public Quaternion rot;
        public string user_path;
        public string interaction_profile;

        public RemoteDevice(uint _id, Vec3 _pos, Quaternion _rot, string _user_path, string _interaction_profile) {
            id = _id;
            if (_pos == null) throw new ArgumentNullException(nameof(_pos));
            pos = _pos;
            if (_rot == null) throw new ArgumentNullException(nameof(_rot));
            rot = _rot;
            if (_user_path == null) throw new ArgumentNullException(nameof(_user_path));
            user_path = _user_path;
            if (_interaction_profile == null) throw new ArgumentNullException(nameof(_interaction_profile));
            interaction_profile = _interaction_profile;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u32(id);
            pos.Serialize(serializer);
            rot.Serialize(serializer);
            serializer.serialize_str(user_path);
            serializer.serialize_str(interaction_profile);
            serializer.decrease_container_depth();
        }

        public int BincodeSerialize(byte[] outputBuffer) => BincodeSerialize(new ArraySegment<byte>(outputBuffer));

        public int BincodeSerialize(ArraySegment<byte> outputBuffer) {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer(outputBuffer);
            Serialize(serializer);
            return serializer.get_buffer_offset();
        }

        public byte[] BincodeSerialize()  {
            Serde.ISerializer serializer = new Bincode.BincodeSerializer();
            Serialize(serializer);
            return serializer.get_bytes();
        }

        public static RemoteDevice Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RemoteDevice obj = new RemoteDevice(
            	deserializer.deserialize_u32(),
            	Vec3.Deserialize(deserializer),
            	Quaternion.Deserialize(deserializer),
            	deserializer.deserialize_str(),
            	deserializer.deserialize_str());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RemoteDevice BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RemoteDevice BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RemoteDevice value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RemoteDevice other && Equals(other);

        public static bool operator ==(RemoteDevice left, RemoteDevice right) => Equals(left, right);

        public static bool operator !=(RemoteDevice left, RemoteDevice right) => !Equals(left, right);

        public bool Equals(RemoteDevice other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!id.Equals(other.id)) return false;
            if (!pos.Equals(other.pos)) return false;
            if (!rot.Equals(other.rot)) return false;
            if (!user_path.Equals(other.user_path)) return false;
            if (!interaction_profile.Equals(other.interaction_profile)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + id.GetHashCode();
                value = 31 * value + pos.GetHashCode();
                value = 31 * value + rot.GetHashCode();
                value = 31 * value + user_path.GetHashCode();
                value = 31 * value + interaction_profile.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RemoteDevice Clone() => (RemoteDevice)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class RemoteConfigurationSnapshot: IEquatable<RemoteConfigurationSnapshot>, ICloneable {
        public uint version;
        public Serde.ValueArray<string> user_paths;
        public Serde.ValueArray<RemoteInteractionProfile> interaction_profiles;
        public string name;

        public RemoteConfigurationSnapshot(uint _version, Serde.ValueArray<string> _user_paths, Serde.ValueArray<RemoteInteractionProfile> _interaction_profiles, string _name) {
            version = _version;
            if (_user_paths == null) throw new ArgumentNullException(nameof(_user_paths));
            user_paths = _user_paths;
            if (_interaction_profiles == null) throw new ArgumentNullException(nameof(_interaction_profiles));
            interaction_profiles = _interaction_profiles;
            if (_name == null) throw new ArgumentNullException(nameof(_name));
            name = _name;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_u32(version);
            TraitHelpers.serialize_vector_str(user_paths, serializer);
            TraitHelpers.serialize_vector_RemoteInteractionProfile(interaction_profiles, serializer);
            serializer.serialize_str(name);
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

        public static RemoteConfigurationSnapshot Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RemoteConfigurationSnapshot obj = new RemoteConfigurationSnapshot(
            	deserializer.deserialize_u32(),
            	TraitHelpers.deserialize_vector_str(deserializer),
            	TraitHelpers.deserialize_vector_RemoteInteractionProfile(deserializer),
            	deserializer.deserialize_str());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RemoteConfigurationSnapshot BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RemoteConfigurationSnapshot BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RemoteConfigurationSnapshot value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RemoteConfigurationSnapshot other && Equals(other);

        public static bool operator ==(RemoteConfigurationSnapshot left, RemoteConfigurationSnapshot right) => Equals(left, right);

        public static bool operator !=(RemoteConfigurationSnapshot left, RemoteConfigurationSnapshot right) => !Equals(left, right);

        public bool Equals(RemoteConfigurationSnapshot other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!version.Equals(other.version)) return false;
            if (!user_paths.Equals(other.user_paths)) return false;
            if (!interaction_profiles.Equals(other.interaction_profiles)) return false;
            if (!name.Equals(other.name)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + version.GetHashCode();
                value = 31 * value + user_paths.GetHashCode();
                value = 31 * value + interaction_profiles.GetHashCode();
                value = 31 * value + name.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RemoteConfigurationSnapshot Clone() => (RemoteConfigurationSnapshot)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary

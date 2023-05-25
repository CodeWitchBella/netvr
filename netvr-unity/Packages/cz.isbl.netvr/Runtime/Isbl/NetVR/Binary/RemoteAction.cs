using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class RemoteAction: IEquatable<RemoteAction>, ICloneable {
        public ActionType type;
        public string name;
        public string localized_name;
        public string binding;

        public RemoteAction(ActionType _type, string _name, string _localized_name, string _binding) {
            if (_type == null) throw new ArgumentNullException(nameof(_type));
            type = _type;
            if (_name == null) throw new ArgumentNullException(nameof(_name));
            name = _name;
            if (_localized_name == null) throw new ArgumentNullException(nameof(_localized_name));
            localized_name = _localized_name;
            if (_binding == null) throw new ArgumentNullException(nameof(_binding));
            binding = _binding;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            type.Serialize(serializer);
            serializer.serialize_str(name);
            serializer.serialize_str(localized_name);
            serializer.serialize_str(binding);
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

        public static RemoteAction Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RemoteAction obj = new RemoteAction(
            	ActionType.Deserialize(deserializer),
            	deserializer.deserialize_str(),
            	deserializer.deserialize_str(),
            	deserializer.deserialize_str());
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RemoteAction BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RemoteAction BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RemoteAction value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RemoteAction other && Equals(other);

        public static bool operator ==(RemoteAction left, RemoteAction right) => Equals(left, right);

        public static bool operator !=(RemoteAction left, RemoteAction right) => !Equals(left, right);

        public bool Equals(RemoteAction other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!type.Equals(other.type)) return false;
            if (!name.Equals(other.name)) return false;
            if (!localized_name.Equals(other.localized_name)) return false;
            if (!binding.Equals(other.binding)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + type.GetHashCode();
                value = 31 * value + name.GetHashCode();
                value = 31 * value + localized_name.GetHashCode();
                value = 31 * value + binding.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RemoteAction Clone() => (RemoteAction)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary

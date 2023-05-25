using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Numerics;

namespace Isbl.NetVR.Binary {

    public sealed class RemoteInteractionProfile: IEquatable<RemoteInteractionProfile>, ICloneable {
        public string path;
        public Serde.ValueArray<RemoteAction> bindings;

        public RemoteInteractionProfile(string _path, Serde.ValueArray<RemoteAction> _bindings) {
            if (_path == null) throw new ArgumentNullException(nameof(_path));
            path = _path;
            if (_bindings == null) throw new ArgumentNullException(nameof(_bindings));
            bindings = _bindings;
        }

        public void Serialize(Serde.ISerializer serializer) {
            serializer.increase_container_depth();
            serializer.serialize_str(path);
            TraitHelpers.serialize_vector_RemoteAction(bindings, serializer);
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

        public static RemoteInteractionProfile Deserialize(Serde.IDeserializer deserializer) {
            deserializer.increase_container_depth();
            RemoteInteractionProfile obj = new RemoteInteractionProfile(
            	deserializer.deserialize_str(),
            	TraitHelpers.deserialize_vector_RemoteAction(deserializer));
            deserializer.decrease_container_depth();
            return obj;
        }

        public static RemoteInteractionProfile BincodeDeserialize(byte[] input) => BincodeDeserialize(new ArraySegment<byte>(input));

        public static RemoteInteractionProfile BincodeDeserialize(ArraySegment<byte> input) {
            if (input == null) {
                 throw new Serde.DeserializationException("Cannot deserialize null array");
            }
            Serde.IDeserializer deserializer = new Bincode.BincodeDeserializer(input);
            RemoteInteractionProfile value = Deserialize(deserializer);
            if (deserializer.get_buffer_offset() < input.Count) {
                 throw new Serde.DeserializationException("Some input bytes were not read");
            }
            return value;
        }
        public override bool Equals(object obj) => obj is RemoteInteractionProfile other && Equals(other);

        public static bool operator ==(RemoteInteractionProfile left, RemoteInteractionProfile right) => Equals(left, right);

        public static bool operator !=(RemoteInteractionProfile left, RemoteInteractionProfile right) => !Equals(left, right);

        public bool Equals(RemoteInteractionProfile other) {
            if (other == null) return false;
            if (ReferenceEquals(this, other)) return true;
            if (!path.Equals(other.path)) return false;
            if (!bindings.Equals(other.bindings)) return false;
            return true;
        }

        public override int GetHashCode() {
            unchecked {
                int value = 7;
                value = 31 * value + path.GetHashCode();
                value = 31 * value + bindings.GetHashCode();
                return value;
            }
        }

        /// <summary>Creates a shallow clone of the object.</summary>
        public RemoteInteractionProfile Clone() => (RemoteInteractionProfile)MemberwiseClone();

        object ICloneable.Clone() => Clone();

    }

} // end of namespace Isbl.NetVR.Binary
